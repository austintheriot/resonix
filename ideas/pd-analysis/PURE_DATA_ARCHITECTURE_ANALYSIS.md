# Pure Data Architecture Analysis for Resonix

## Overview
Pure Data (Pd) is a mature audio/visual programming environment with 25+ years of development. This analysis examines its core architectural patterns, particularly relevant to Resonix's design decisions around buffer management, DSP processing pipelines, I/O isolation, and modularity.

---

## 1. MEMORY ALLOCATION AND BUFFER MANAGEMENT

### 1.1 Unified Allocation Strategy
Pure Data uses a two-tier memory management system:

**File**: `/src/m_pd.h` (lines 362-366)
```c
EXTERN void *getbytes(size_t nbytes);
EXTERN void *getzbytes(size_t nbytes);  /* zeroed bytes */
EXTERN void *copybytes(const void *src, size_t nbytes);
EXTERN void freebytes(void *x, size_t nbytes);
EXTERN void *resizebytes(void *x, size_t oldsize, size_t newsize);
```

**Key Pattern**: Sized allocation/deallocation
- All allocations include explicit size tracking
- Enables runtime verification and leak detection
- Paired size parameters prevent silent corruption
- Implementation allows hook-based memory monitoring

**Decision Driver**: Pd runs long-lived sessions where memory leaks are unacceptable. External objects must participate in the same tracking system.

### 1.2 Signal Buffer Pre-Allocation with Reuse Pool
Audio signals use a sophisticated buffer management system with reuse pools.

**File**: `/src/d_ugen.c` (lines 511-567)
```c
t_signal *signal_new(int length, int nchans, t_float sr, t_sample *scalarptr)
{
    int allocsize = 0;
    t_signal *ret, **whichlist;
    if (sr < 1)
        bug("signal_new");
    if (length && !scalarptr)
    {
        int logn = ilog2(length*nchans);      /* Calculate power-of-2 */
        if ((1<<logn) < length*nchans)
            logn++;
        allocsize = 1<<logn;                   /* Round up to nearest power-of-2 */
        whichlist = THIS->u_freelist + logn;
    }
    else /* scalar or borrowed signal */
        whichlist = &THIS->u_freeborrowed;

    /* Try to reclaim from free list */
    if ((ret = *whichlist))
        *whichlist = ret->s_nextfree;
    else
    {
        ret = (t_signal *)t_getbytes(sizeof *ret);
        if (allocsize)
            ret->s_vec = (t_sample *)getbytes(allocsize * sizeof (*ret->s_vec));
        ret->s_nextused = THIS->u_signals;
        THIS->u_signals = ret;
    }
    /* ... initialize signal metadata ... */
    return (ret);
}
```

**Architecture Insights**:

1. **Power-of-2 Binning**: Buffers sized to powers of 2 (256 samples, 512, 1024, etc.)
   - Reduces fragmentation
   - Enables efficient size-based lookup in freelists
   - 34 freelists (MAXLOGSIG+1, typically 0-32)

2. **Three-Tier Signal Types**:
   - **Owned signals**: Own their buffer memory, tracked in size-based freelists
   - **Borrowed signals**: Share buffer from another signal, tracked separately
   - **Scalar signals**: Point to a persistent value (not allocated)

3. **Buffer Lifecycle**:
   ```
   signal_new() -> (allocate or reclaim) -> use -> signal_makereusable() 
   -> (add to freelist) -> reuse or cleanup
   ```

4. **Reference Counting**: Each signal has `s_refcount` to track how many objects use it
   - Prevents premature reuse
   - Signals marked reusable only when refcount reaches zero

5. **Cleanup on DSP Stop**: `signal_cleanup()` (lines 429-443)
   - All buffers freed at once
   - Predictable memory behavior
   - No per-block allocation during DSP

### 1.3 I/O Buffer Strategy
**File**: `/src/s_audio.c` (lines 82-131)

Audio I/O buffers are fixed-size, pre-allocated per channel:
```c
void sys_setchsr(int chin, int chout, int sr)
{
    int oldinbytes = (oldchin ? oldchin : 2) * (DEFDACBLKSIZE*sizeof(t_sample));
    int oldoutbytes = (oldchout ? oldchout : 2) * (DEFDACBLKSIZE*sizeof(t_sample));
    
    if (!STUFF->st_soundin || chin != oldchin)
    {
        if (STUFF->st_soundin)
            freebytes(STUFF->st_soundin, oldinbytes);
        STUFF->st_soundin = (t_sample *)getbytes(inbytes);
        STUFF->st_inchannels = chin;
    }
    memset(STUFF->st_soundin, 0, inbytes);
    /* ... similar for output ... */
}
```

**Pattern**:
- Single pre-allocated ring buffer per I/O direction
- Size: `channels * DEFDACBLKSIZE * sizeof(t_sample)`
- DEFDACBLKSIZE = 64 samples (typical)
- Reallocated only when channel count changes
- Zeroed on every reallocation (defensive)

---

## 2. DSP PROCESSING PIPELINE ARCHITECTURE

### 2.1 The DSP Chain: Linked List of Operations
Pure Data builds a linear, flattened DSP chain at graph compilation time.

**Data Structure** (from `/src/d_ugen.c`, lines 35-48):
```c
struct _instanceugen
{
    t_int *u_dspchain;         /* Linear array of operations */
    int u_dspchainsize;        /* Total number of elements */
    t_signal *u_signals;       /* All signal objects */
    int u_sortno;              /* Graph compilation version number */
    t_signal *u_freelist[MAXLOGSIG+1];  /* Reuse pools by size */
    t_signal *u_freeborrowed;   /* Borrowed signals pool */
    int u_phase;                /* Current tick within block */
};
```

### 2.2 DSP Chain Construction
**File**: `/src/d_ugen.c` (lines 365-401)

Two APIs for adding operations:
```c
void dsp_add(t_perfroutine f, int n, ...)
{
    /* Variadic: dsp_add(function, arg_count, arg1, arg2, ...); */
    int newsize = THIS->u_dspchainsize + n+1;
    THIS->u_dspchain = t_resizebytes(THIS->u_dspchain,
        THIS->u_dspchainsize * sizeof (t_int), newsize * sizeof (t_int));
    THIS->u_dspchain[THIS->u_dspchainsize-1] = (t_int)f;
    va_start(ap, n);
    for (i = 0; i < n; i++) {
        THIS->u_dspchain[THIS->u_dspchainsize + i] = va_arg(ap, t_int);
    }
    va_end(ap);
    THIS->u_dspchain[newsize-1] = (t_int)dsp_done;
    THIS->u_dspchainsize = newsize;
}

void dsp_addv(t_perfroutine f, int n, t_int *vec)
{
    /* Vectorized version for efficiency */
    int newsize = THIS->u_dspchainsize + n+1;
    THIS->u_dspchain = t_resizebytes(...);
    THIS->u_dspchain[THIS->u_dspchainsize-1] = (t_int)f;
    for (i = 0; i < n; i++)
        THIS->u_dspchain[THIS->u_dspchainsize + i] = vec[i];
    THIS->u_dspchain[newsize-1] = (t_int)dsp_done;
    THIS->u_dspchainsize = newsize;
}
```

### 2.3 DSP Execution Loop
**File**: `/src/d_ugen.c` (lines 403-411)

```c
void dsp_tick(void)
{
    if (THIS->u_dspchain) {
        t_int *ip;
        for (ip = THIS->u_dspchain; ip; ) 
            ip = (*(t_perfroutine)(*ip))(ip);
        THIS->u_phase++;
    }
}
```

**Chain Termination**: Each routine returns pointer to next operation, or NULL to end.

### 2.4 Block~ for Multi-Rate Processing
The `block~` object implements nested processing at different block sizes/rates.

**File**: `/src/d_ugen.c` (lines 130-151)
```c
typedef struct _block
{
    t_object x_obj;
    int x_calcsize;     /* samples to compute per invocation */
    int x_overlap;      /* overlapping factor */
    int x_phase;        /* phase within parent block period */
    int x_period;       /* submultiple of parent */
    int x_frequency;    /* supermultiple of parent */
    int x_chainonset;   /* where in DSP chain */
    int x_blocklength;  /* length of subgraph */
    int x_epiloglength;
    char x_switched;    /* switch~/block~ distinction */
    char x_switchon;
    char x_reblock;     /* inlet/outlet reblocking */
    int x_upsample;
    int x_downsample;
    int x_offset;
} t_block;
```

**Architecture Pattern**:
- Prolog code runs before block (lines 301-321)
- Checks if this phase should execute
- If not, skips to epilog
- Epilog (lines 323-338) handles upsampling/downsampling
- Allows arbitrary nesting depth

### 2.5 Signal Dependency Resolution
Pure Data doesn't have explicit topological sorting in the traditional sense. Instead:

1. **Graph Compilation** (`ugen_start_graph`, line 744+): 
   - Walks the patch graph
   - Creates temporary ugenbox structures
   - Records signal connections

2. **Object Ordering**: Implicit in how objects are traversed
   - Objects output signals
   - Connected objects read signals
   - No cycle detection (user's responsibility)

3. **Reblocking**: When vector sizes differ
   - Automatic scalar-to-vector promotion
   - Overlapping buffers for rate changes
   - Configured per inlet/outlet

---

## 3. I/O SEPARATION PATTERNS

### 3.1 Audio I/O Objects (adc~/dac~)
**File**: `/src/d_dac.c` (lines 45-88)

```c
static void dac_dsp(t_dac *x, t_signal **sp)
{
    t_int i, j;
    for (i = 0; i < x->x_n; i++) {
        int ch = (int)(x->x_vec[i] - 1);
        if (sp[i]->s_length != DEFDACBLKSIZE)
            pd_error(x, "dac~: vector size mismatch...");
        else for (j = 0; j < sp[i]->s_nchans; j++) {
            if (ch + j >= 0 && ch + j < sys_get_outchannels())
                dsp_add(plus_perform, 4,
                    STUFF->st_soundout + DEFDACBLKSIZE * (ch + j),
                    sp[i]->s_vec + j * sp[i]->s_length,
                    STUFF->st_soundout + DEFDACBLKSIZE * (ch + j),
                    (t_int)DEFDACBLKSIZE);
        }
    }
}
```

**Architecture Insights**:

1. **I/O Buffers**: Kept in global `STUFF->st_soundout/st_soundin`
2. **Channel Mapping**: Objects specify which hardware channels to use
3. **DSP Operation**: `dsp_add(plus_perform, ...)` = sum patch output to I/O buffer
   - adc~ performs copy from input buffer to patch signal
   - dac~ performs sum to output buffer (accumulate mode, not replace)
4. **Multichannel Support**: CLASS_MULTICHANNEL flag enables variable channels

### 3.2 Message vs Signal Separation
**File**: `/src/m_pd.h` (lines 698-750+)

Two completely separate domains:
- **Message domain**: Inlets/outlets, event-driven, sample-accurate
- **Signal domain**: DSP chain, block-based, fixed rate

**Definition**:
```c
typedef void (*t_method)(void);  /* message handler */
typedef t_int *(*t_perfroutine)(t_int *args);  /* DSP operation */
```

Objects can implement both simultaneously:
- Message inlets always available
- Signal inlets created with `signalinlet_new()`
- Control rate messages at audio block boundaries

### 3.3 Scheduler Integration
**File**: `/src/s_inter.c` (top of file)

Two event loops run in one thread:
1. **Message scheduler**: Processes control messages
2. **Audio scheduler**: Calls `dsp_tick()` at block rate

Integration points:
- Audio callback triggers message processing
- Messages queued from GUI thread use lock-free ringbuffer
- Single-threaded Pd avoids complex synchronization

---

## 4. MODULARITY AND PORTABILITY PATTERNS

### 4.1 Abstract Audio Interface
**File**: `/src/s_stuff.h` (lines 67-271)

Platform abstraction layer with multiple implementations:

```c
#define API_NONE 0
#define API_ALSA 1
#define API_OSS 2
#define API_MMIO 3
#define API_PORTAUDIO 4
#define API_JACK 5
/* ... etc ... */

/* Abstract interface */
int pa_open_audio(int inchans, int outchans, int rate, 
                  t_sample *soundin, t_sample *soundout, 
                  int framesperbuf, int nbuffers,
                  int indeviceno, int outdeviceno, 
                  t_audiocallback callback);
void pa_close_audio(void);
int pa_send_dacs(void);

/* Same pattern for: oss_, alsa_, jack_, mmio_, audiounit_, etc. */
```

**Pattern**:
- Single `sys_send_dacs()` public interface
- Implementation selected at runtime
- Each API provides: `open_audio`, `close_audio`, `send_dacs`, `reportidle`, `getdevs`
- Decouples core Pd from audio infrastructure

### 4.2 External Object System
**File**: `/src/s_loader.c` (lines 76-140)

```c
static const char*sys_dllextent_base[] = {
#if defined(__linux__)
    ".l_amd64", ".l_i386", ".l_arm", ".l_arm64",
    ".pd_linux",
#elif defined(__APPLE__)
    ".d_amd64", ".d_i386", ".d_ppc", ".d_arm64",
    ".d_fat", ".pd_darwin",
#elif defined(_WIN32)
    ".m_amd64", ".m_i386", SYSTEMEXT,
#endif
    0
};
```

**Plugin Architecture**:

1. **Class Registration**: Objects export a `*_setup()` function
   ```c
   void d_dac_setup(void) {
       dac_setup();
       adc_setup();
   }
   ```

2. **Dynamic Loading**: 
   - Searches for `.pd_linux`, `.d_darwin`, `.m_amd64`, etc.
   - Falls back to generic extension
   - Allows fat binaries, architecture-specific versions

3. **Symbol Resolution**: Standard dlopen/dlsym

4. **Versioning**: Runtime version check available
   ```c
   int pd_compatibilitylevel;  /* e.g., 43 for pd 0.43 compatibility */
   ```

### 4.3 Internal Structure Isolation
**File**: `/src/m_pd.h` (opaque struct forward declarations)

```c
EXTERN_STRUCT _outlet;      /* Not defined in .h */
#define t_outlet struct _outlet

EXTERN_STRUCT _inlet;
#define t_inlet struct _inlet

EXTERN_STRUCT _binbuf;
#define t_binbuf struct _binbuf
```

**Pattern**: Public API exposes only pointers, hides implementation
- Allows internal changes without breaking externals
- Objects access via functions (`outlet_new()`, `inlet_new()`)
- Not direct struct member access

### 4.4 Cross-Platform Abstractions
**File**: `/src/s_stuff.h`, `/src/s_inter.c`

```c
/* File I/O */
EXTERN int sys_open(const char *path, int oflag, ...);
EXTERN int sys_close(int fd);
EXTERN FILE *sys_fopen(const char *filename, const char *mode);
EXTERN int sys_fclose(FILE *stream);

/* Time */
EXTERN double sys_getrealtime(void);
EXTERN int sched_geteventno(void);
EXTERN double clock_getlogicaltime(void);

/* Threading */
EXTERN void sys_lock(void);
EXTERN void sys_unlock(void);
EXTERN int sys_trylock(void);

/* Platform-specific threading */
#define PERTHREAD __thread   /* on Unix */
#define PERTHREAD __declspec(thread)  /* on MSVC */
```

---

## 5. KEY ARCHITECTURAL PATTERNS

### 5.1 Class Definition and Method Dispatch
**File**: `/src/m_imp.h` (lines 37-70)

```c
struct _class {
    t_symbol *c_name;
    t_symbol *c_helpname;
    size_t c_size;
    t_methodentry *c_methods;      /* method table */
    int c_nmethod;
    t_method c_freemethod;
    t_bangmethod c_bangmethod;     /* quick paths for common msgs */
    t_pointermethod c_pointermethod;
    t_floatmethod c_floatmethod;
    t_symbolmethod c_symbolmethod;
    t_listmethod c_listmethod;
    t_anymethod c_anymethod;       /* catch-all for unknown messages */
    const struct _widgetbehavior *c_wb;
    int c_floatsignalin;
    unsigned int c_multichannel:1;
    unsigned int c_nopromotesig:1;
    unsigned int c_nopromoteleft:1;
};
```

**Design**:
1. Common message types (bang, float, symbol, list) have dedicated slots
2. Generic messages use hash-table lookup
3. Enables fast path for frequent operations
4. Slower path for rare/user-defined messages

### 5.2 Signal Processing Method Pattern
**File**: `/src/d_arithmetic.c` (lines 185-195)

```c
plus_class = class_new(gensym("+~"), (t_newmethod)plus_new, 0,
    sizeof(t_plus), 0, A_DEFFLOAT, 0);
CLASS_MAINSIGNALIN(plus_class, t_plus, x_f);
class_addmethod(plus_class, (t_method)plus_dsp, gensym("dsp"), A_CANT, 0);
```

**Pattern**:
1. `dsp_add()` is called during DSP graph compilation (not runtime)
2. Each object's `dsp` method is called exactly once per graph build
3. Objects register operations via `dsp_add()`, not by polling

### 5.3 Error Handling and Logging
**File**: `/src/m_pd.h` (lines 643-658)

```c
EXTERN void pd_error(const void *object, const char *fmt, ...);
EXTERN void logpost(const void *object, int level, const char *fmt, ...);

typedef enum {
    PD_CRITICAL = 0,
    PD_ERROR,
    PD_NORMAL,
    PD_DEBUG,
    PD_VERBOSE
} t_loglevel;
```

**Pattern**: 
- Errors printed with object context
- Levels enable log filtering
- No exceptions (pure C)
- Fail-safe approach: post error, continue

### 5.4 Versioning and Compatibility
**File**: `/src/m_pd.h` (lines 11-24)

```c
#define PD_MAJOR_VERSION 0
#define PD_MINOR_VERSION 56
#define PD_BUGFIX_VERSION 2

#define PD_VERSION(major, minor, bugfix) \
    (((major) << 16) + ((minor) << 8) + ((bugfix) > 255 ? 255 : (bugfix)))
#define PD_VERSION_CODE PD_VERSION(PD_MAJOR_VERSION, PD_MINOR_VERSION, PD_BUGFIX_VERSION)

extern int pd_compatibilitylevel;

/* Externals can check: */
#if PD_VERSION_CODE < PD_VERSION(0, 56, 0)
    /* use legacy code */
#endif
```

---

## 6. RELEVANT ARCHITECTURAL DECISIONS FOR RESONIX

### 6.1 What Resonix Should Adopt
1. **Fixed Block Size DSP Chain**: Pre-compile operations to linear chain
   - Cache-friendly
   - No dynamic allocation per block
   - Predictable latency

2. **Power-of-2 Signal Buffer Sizing**: Reduces memory fragmentation
   - Efficient allocation/deallocation
   - Size-based reuse pools

3. **Borrowed Signals Pattern**: Allow multiple objects to share one buffer
   - Reference counting prevents premature reuse
   - Transparent memory ownership

4. **Separation of Control and Audio Domains**:
   - Message processing ≠ DSP processing
   - Audio domain is deterministic
   - Control domain can block without affecting audio

5. **Abstract I/O Interface**: Multiple implementations selectable at runtime
   - Supports JACK, ALSA, PortAudio, AudioUnit, WinMM

6. **Sized Allocation Tracking**: Every `malloc` includes explicit size
   - Enables leak detection
   - Supports memory monitoring
   - Required for long-running services

### 6.2 What Resonix Should Avoid
1. **Dynamic Graph Changes During DSP**: Pd requires full graph recompilation
   - Causes DSP discontinuities
   - Makes real-time guarantees difficult
   
2. **Circular Signal Dependencies**: Not detected or handled
   - User's responsibility to avoid
   - Can crash silently

3. **Variable Block Sizes in Hot Path**: Reblocking adds overhead
   - Better to standardize block size across graph

4. **Per-Block Memory Allocation**: Audio callback should not malloc
   - Pd pre-allocates everything at graph compile time

### 6.3 Questions for Resonix Design
1. **Block-size negotiation**: How should Resonix handle upstream/downstream mismatches?
   - Pd uses automatic reblocking (expensive)
   - Could mandate block size

2. **Real-time thread handling**: Pd single-threads; Resonix plans callbacks
   - Need explicit lock-free data structures
   - Ringbuffers for cross-thread communication

3. **Graph compilation**: How often should DSP chain be rebuilt?
   - Pd does full rebuild (slow but safe)
   - Resonix could support live editing with atomic swaps

4. **Error recovery**: Pd posts errors but continues
   - Should Resonix halt on audio glitches?
   - Depends on use case (DAW vs plugin)

---

## CODEBASE STATISTICS

- **Total source files**: 137 C/H files in /src
- **Core DSP logic**: ~10,000 lines (d_*.c)
- **Scheduler/I/O**: ~5,000 lines (s_*.c)
- **Data structures**: ~3,000 lines (m_*.c)
- **Age**: Original code from 1997 (25+ years)
- **Language**: Pure C, minimal dependencies

---

## REFERENCE IMPLEMENTATIONS

To understand specific patterns in detail:

1. **Signal allocation**: `/src/d_ugen.c` lines 511-567
2. **DSP chain building**: `/src/d_ugen.c` lines 365-401
3. **Block execution**: `/src/d_ugen.c` lines 403-411
4. **I/O integration**: `/src/d_dac.c` (5.8KB, very clean)
5. **Multi-rate processing**: `/src/d_ugen.c` lines 100-356
6. **Class registration**: `/src/d_arithmetic.c` (arithmetic operators)
7. **Audio API abstraction**: `/src/s_stuff.h` (API definitions)
8. **External loading**: `/src/s_loader.c` (dynamic linking)

