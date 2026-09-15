#ifndef DMD_CORE_H
#define DMD_CORE_H

#include <stdint.h>

/* Independent of event protocol and recording format versions. */
#define DMD_CORE_ABI_VERSION UINT32_C(1)
#define DMD_OK INT32_C(0)
#define DMD_END INT32_C(1)
#define DMD_ERROR INT32_C(-1)
#define DMD_FORMAT_REAL_F32 UINT32_C(1)
#define DMD_FORMAT_IQ_F32 UINT32_C(2)
#define DMD_TEXT_INFO UINT32_C(1)
#define DMD_TEXT_LIST UINT32_C(2)
#define DMD_TEXT_MASK UINT32_C(3)
#define DMD_DIAG_VERSION_CONFLICT UINT32_C(1)
#define DMD_DIAG_AMBIGUOUS_NAME UINT32_C(2)

#ifdef __cplusplus
extern "C" {
#endif

/* All integers have the stated widths. Structures use the platform C ABI layout
 * and natural alignment; no packing. All strings are UTF-8 byte spans without a
 * terminator; input paths must not contain NUL. A zero-length span may be NULL.
 * Nonempty spans must be valid and remain accessible for their stated length.
 * No STL types or exceptions cross this boundary. Functions report errors using
 * DMD_ERROR and an optional borrowed diagnostic; they never throw or unwind.
 *
 * Handles are opaque, non-NULL on successful creation and NULL on failure.
 * Each handle has one owner, is used serially on its creating thread, and is
 * released exactly once by its matching close/destroy function. NULL destruction
 * is a no-op. Input spans are borrowed only for the duration of a call. Returned
 * spans are immutable core-owned memory, valid until the next call on that handle
 * or its destruction. Callers must copy them to retain them. No caller frees a
 * returned span; no function takes ownership of caller memory.
 *
 * Status is DMD_OK, DMD_END (only documented iteration/drain operations), or
 * DMD_ERROR. Every non-NULL output is initialized, even on error. Diagnostic
 * messages follow the same lifetime as other returned spans. */
typedef struct DmdSignal DmdSignal;
typedef struct DmdCatalogue DmdCatalogue;
typedef struct DmdRun DmdRun;
typedef struct DmdBytes { const uint8_t *data; uint64_t len; } DmdBytes;
typedef struct DmdError { int32_t code; DmdBytes message; } DmdError;
typedef struct DmdStream {
    uint32_t format;
    uint32_t channels;
    uint64_t rate_numerator;
    uint64_t rate_denominator;
} DmdStream;
typedef struct DmdBlock {
    const float *samples;
    uint64_t frames;
    uint64_t source_position;
} DmdBlock;
typedef struct DmdModule {
    DmdBytes id;
    DmdBytes name;
    DmdBytes version;
} DmdModule;
typedef struct DmdDiagnostic {
    uint32_t kind;
    DmdBytes name;
    DmdBytes message;
} DmdDiagnostic;
typedef struct DmdParameter { DmdBytes name; DmdBytes value; } DmdParameter;
typedef struct DmdResult { DmdBytes identifier; DmdBytes payload; } DmdResult;

uint32_t dmd_core_abi_version(void);

/* Open decodes stream characteristics. Source and output characteristics are
 * separate: output initially equals source. Rates are positive rational frames
 * per second with a nonzero denominator. Supported formats are native-endian
 * IEEE-754 float32: REAL is one scalar per channel per frame, IQ is interleaved
 * I then Q per channel per frame. Channels are interleaved within each frame.
 * A source position is always a frame index in the source file, including after
 * conversion or resampling; one IQ frame contains an I/Q pair per channel.
 * Errors returned by failed open have thread-local lifetime until the next open.
 * Target format, channel count and rate must be set before the first read. */
int32_t dmd_signal_open(DmdBytes path, DmdSignal **out, DmdError *error);
void dmd_signal_close(DmdSignal *signal);
int32_t dmd_signal_source(DmdSignal *signal, DmdStream *out, DmdError *error);
int32_t dmd_signal_target(DmdSignal *signal, const DmdStream *target, DmdError *error);
int32_t dmd_signal_output(DmdSignal *signal, DmdStream *out, DmdError *error);
/* Read returns a nonempty prepared output block or DMD_END at source exhaustion.
 * Finish stops source consumption and flushes conversion. Then drain returns
 * remaining nonempty blocks until DMD_END. Finish is called once; read/target are
 * invalid afterwards. Close is valid in every state, including after errors. */
int32_t dmd_signal_read(DmdSignal *signal, DmdBlock *out, DmdError *error);
int32_t dmd_signal_finish(DmdSignal *signal, DmdError *error);
int32_t dmd_signal_drain(DmdSignal *signal, DmdBlock *out, DmdError *error);

/* Open snapshots modules over the ordered path list. Enumeration order is stable
 * within that snapshot; IDs uniquely identify descriptors in it. Duplicate names
 * and conflicting versions produce diagnostics, never silent selection. IDs,
 * descriptors and text are not paths. text supplies UTF-8 responses for info/list/
 * mask; LIST accepts an empty ID, INFO and MASK require a unique descriptor ID.
 * Index iteration returns DMD_END when exhausted. A failed open's error has
 * thread-local lifetime until the next catalogue open. */
int32_t dmd_catalogue_open(const DmdBytes *paths, uint64_t count, DmdCatalogue **out, DmdError *error);
void dmd_catalogue_close(DmdCatalogue *catalogue);
int32_t dmd_catalogue_module(DmdCatalogue *catalogue, uint64_t index, DmdModule *out, DmdError *error);
int32_t dmd_catalogue_text(DmdCatalogue *catalogue, uint32_t kind, DmdBytes id, DmdBytes *out, DmdError *error);
int32_t dmd_catalogue_diagnostic(DmdCatalogue *catalogue, uint64_t index, DmdDiagnostic *out, DmdError *error);

/* Create chooses an exact descriptor ID, copies parameters and stream description,
 * and returns an independent instance that may outlive the catalogue. Ambiguous
 * selection is an error. Parameter names/values are UTF-8; duplicate names fail.
 * Submit consumes the borrowed samples synchronously using the declared stream
 * layout; source_position is a source frame index, never an output frame index.
 * Results are identifier/payload byte spans, with UTF-8 identifiers and opaque
 * payloads. This contract specifies no event encoding or result-file layout.
 * Next returns DMD_END if the queue is currently empty. Finalize is called once,
 * disallows further submissions and queues final results; drain with next until
 * DMD_END, then destroy. Destroy is also valid before finalize or after an error. */
int32_t dmd_run_create(DmdCatalogue *catalogue, DmdBytes id, const DmdParameter *parameters, uint64_t count, const DmdStream *stream, DmdRun **out, DmdError *error);
int32_t dmd_run_submit(DmdRun *run, const DmdBlock *block, DmdError *error);
int32_t dmd_run_next(DmdRun *run, DmdResult *out, DmdError *error);
int32_t dmd_run_finalize(DmdRun *run, DmdError *error);
void dmd_run_destroy(DmdRun *run);

#ifdef __cplusplus
}
#endif
#endif
