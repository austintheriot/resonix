# Pure Data: Sample Rate Architecture

A technical summary of how Pure Data stores, propagates, and resamples audio at different sample rates.

---

## Global Sample Rate Storage

The sample rate is stored per-instance in `struct _instancestuff` (`src/s_stuff.h`):

```c
struct _instancestuff {
    t_float st_dacsr;   /* I/O sample rate */
    ...
};
```

Accessed via the `STUFF->st_dacsr` macro. The default is 48000 Hz (`DEFAULTSRATE`, `src/m_pd.h:131`).

**Key accessor:**
```c
t_float sys_getsr(void) { return STUFF->st_dacsr; }  // s_audio.c:575
```

---

## When and How Sample Rate Changes

The audio backend calls `sys_setchsr(int chin, int chout, int sr)` (`src/s_audio.c:82`) whenever the channel count or sample rate changes. This function:

1. Updates `STUFF->st_dacsr = sr`
2. Reallocates input/output channel buffers if channel counts changed
3. **Triggers a full DSP graph rebuild** via `canvas_update_dsp()`

```c
// s_audio.c
void sys_setchsr(int chin, int chout, int sr) {
    ...
    if (STUFF->st_dacsr != sr) changed = 1;
    STUFF->st_dacsr = sr;
    ...
    canvas_update_dsp();   // rebuild the entire DSP graph
}
```

There is no incremental update — a sample rate change always rebuilds the whole DSP chain from scratch.

---

## DSP Graph Rebuild

`canvas_update_dsp()` (`src/g_canvas.c:1504`) tears down the old chain and builds a new one:

```
canvas_update_dsp()
  → canvas_stop_dsp()   // destroy old DSP chain
  → canvas_start_dsp()  // rebuild from scratch
      → ugen_start()
      → canvas_dodsp() for each root canvas
          → ugen_start_graph()   // create DSP context
          → ugen_add()           // register DSP-capable objects
          → ugen_connect()       // record signal connections
          → ugen_done_graph()    // sort objects, allocate signals, compile chain
```

During `ugen_done_graph()` (`src/d_ugen.c:1063`), each signal is allocated with the current sample rate baked in. The compiled DSP chain is a flat array of function pointers + arguments (`u_dspchain`) that is walked every audio tick.

---

## Signals Carry Their Own Sample Rate

Each signal in the graph has its own `s_sr` field (`src/m_pd.h:708`):

```c
typedef struct _signal {
    int s_length;       /* samples per block */
    t_sample *s_vec;    /* sample data */
    t_float s_sr;       /* samples per second */
    int s_nchans;
    int s_overlap;
    ...
} t_signal;
```

`signal_new(int length, int nchans, t_float sr, ...)` sets `s_sr` at creation time. When signals are connected/borrowed between objects, `s_sr` is copied from the source signal.

This means sample rate is a property of the signal, not just a global — which is what enables sub-patches to run at different rates.

---

## Different Rates in Sub-patches: `block~` and `switch~`

The primary mechanism for running a sub-patch at a different rate is the `block~` object (`src/d_ugen.c:132`). It carries:

```c
typedef struct _block {
    int x_upsample;    /* upsampling factor relative to parent */
    int x_downsample;  /* downsampling factor relative to parent */
    int x_overlap;     /* block overlap factor */
    int x_calcsize;    /* local block size */
    char x_reblock;    /* true if inlet/outlet reblocking is needed */
    char x_switched;   /* true if acting as switch~ */
    ...
} t_block;
```

When `ugen_done_graph()` processes a canvas that contains a `block~`, it calculates the local sample rate:

```c
// d_ugen.c:1143
srate = parent_srate * realoverlap * upsample / downsample;
```

If the local rate differs from the parent, `reblock = 1` is set, and resampling functions are inserted into the DSP chain at the sub-patch's inlets and outlets.

`canvas_getsr(t_canvas *x)` (`src/d_ugen.c:222`) walks the canvas hierarchy and accumulates upsample/downsample factors to compute the effective sample rate at any nesting depth:

```c
t_float canvas_getsr(t_canvas *x) {
    t_float srate = sys_getsr();
    for (canvas = x; canvas; canvas = canvas->gl_owner)
        // find block~ and multiply srate by upsample/downsample
    return srate;
}
```

---

## Resampling Implementation

Resampling is handled in `src/d_resample.c`. The `t_resample` struct (`src/m_pd.h:770`) holds the resampling state: up/down factors, buffer, and filter coefficients.

`resample_dsp()` (`src/d_resample.c:125`) decides which resampling perform routine to insert into the DSP chain based on the ratio of input to output buffer sizes:

```c
void resample_dsp(t_resample *x,
                  t_sample *in,  int insize,
                  t_sample *out, int outsize, int method) {
    if (insize == outsize) return;   // no resampling needed

    if (insize > outsize)            // downsampling
        dsp_add(downsampling_perform_0, ...);
    else                             // upsampling
        dsp_add(upsampling_perform_*, ...);
}
```

### Resampling Methods

| Direction | Method | Description |
|-----------|--------|-------------|
| Downsampling | Zero-order hold (default) | Take every Nth sample, skip the rest |
| Upsampling | Zero padding (default) | Insert zeros between samples |
| Upsampling | Sample & hold (`method=1`) | Repeat last sample |
| Upsampling | Linear interpolation (`method=2`) | Lerp between adjacent samples |

Downsampling is always zero-order hold — there is no anti-aliasing filter built in. Upsampling offers three choices via the `block~` argument.

Only integer ratios are supported. Non-integer ratios (e.g., 44100 → 48000) are **not supported** within the DSP graph; those conversions happen at the audio driver level before samples reach Pd.

---

## Runtime Execution

Every scheduler tick calls `dsp_tick()` (`src/d_ugen.c:403`):

```c
void dsp_tick(void) {
    t_int *ip;
    for (ip = THIS->u_dspchain; ip; )
        ip = (*(t_perfroutine)(*ip))(ip);  // walk the compiled chain
    THIS->u_phase++;
}
```

The DSP chain is a flat array. Each entry is a function pointer followed by its arguments (signal buffers, sizes, etc.). Resampling perform routines are interspersed inline at sub-patch boundaries. `block~`-governed sub-patches use a period/frequency counter so they only run every N parent ticks (downsampling) or N times per parent tick (upsampling).

---

## Architectural Summary

```
Audio driver (e.g., JACK/CoreAudio/ALSA)
  │  reports sample rate via sys_setchsr()
  ▼
STUFF->st_dacsr                      ← global single source of truth
  │  triggers canvas_update_dsp()
  ▼
DSP graph rebuild
  │  ugen_done_graph() allocates t_signal objects with s_sr set
  │  block~ objects inject up/downsample factors per sub-patch
  │  resample_dsp() inserts resampling perform routines at boundaries
  ▼
u_dspchain[] (flat array of function ptrs)
  │  walked every audio tick by dsp_tick()
  │  resampling happens inline, no separate pass
  ▼
STUFF->st_soundout[]                 ← output buffer back to driver
```

**Key design decisions:**

- Sample rate is stored in the signal, not just globally — sub-patches can run at different effective rates.
- A sample rate change always triggers a complete DSP graph rebuild. There is no hot-swap.
- Resampling between sub-patches uses simple integer ratios only; no arbitrary rate conversion.
- The DSP chain is a compiled flat array walked in order, not a dynamic graph traversal at runtime — this is what makes Pd's DSP deterministic and low-overhead.

---

## Key Files and Functions

| Function | File | Purpose |
|---|---|---|
| `sys_getsr()` | `src/s_audio.c:575` | Get global sample rate |
| `sys_setchsr()` | `src/s_audio.c:82` | Set sample rate, trigger rebuild |
| `canvas_update_dsp()` | `src/g_canvas.c:1504` | Stop + restart DSP chain |
| `canvas_start_dsp()` | `src/g_canvas.c:1459` | Build DSP chain for all canvases |
| `canvas_dodsp()` | `src/g_canvas.c:1409` | Recursively build DSP for one canvas |
| `ugen_done_graph()` | `src/d_ugen.c:1063` | Sort objects, compile DSP chain |
| `canvas_getsr()` | `src/d_ugen.c:222` | Get effective sample rate at a canvas depth |
| `signal_new()` | `src/d_ugen.c:511` | Allocate signal with embedded `s_sr` |
| `resample_dsp()` | `src/d_resample.c:125` | Insert resampling into DSP chain |
| `dsp_tick()` | `src/d_ugen.c:403` | Execute one block of the DSP chain |
| `dsp_add()` | `src/d_ugen.c:365` | Append a perform routine to the DSP chain |
