#!/bin/bash

cd "$(dirname "$0")"

EXTRA_FLAGS=
if [[ "$1" == "msvc14" ]] ; then
    EXTRA_FLAGS="-use-msvc-mangling --target=x86_64-pc-win32 -DWIN32=1"
    EXTRA_FLAGS="$EXTRA_FLAGS -fms-compatibility-version=19.00"
    EXTRA_FLAGS="$EXTRA_FLAGS -DEXPORT_JS_API=1 -D_CRT_USE_BUILTIN_OFFSETOF"
    EXTRA_FLAGS="$EXTRA_FLAGS -fvisibility=hidden"
fi

: ${BINDGEN:=../../rust-bindgen/target/debug/bindgen}

if [[ ! -x "$BINDGEN" ]]; then
    echo "error: BINDGEN does not exist or isn't executable!"
    echo "error: with BINDGEN=$BINDGEN"
    exit 1
fi

$BINDGEN \
  ${EXTRA_FLAGS} \
  -no-class-constants \
  -no-type-renaming \
  -blacklist-type DefaultHasher \
  -blacklist-type Heap \
  -blacklist-type AutoHashMapRooter \
  -blacklist-type AutoHashSetRooter \
  -blacklist-type TypeIsGCThing \
  -blacklist-type HashMap \
  -blacklist-type HashSet \
  -blacklist-type HashTable \
  -blacklist-type HashTableEntry \
  -blacklist-type AutoStableStringChars \
  -blacklist-type ErrorReport \
  -blacklist-type MemProfiler \
  -opaque-type RuntimeStats \
  -opaque-type EnumeratedArray \
  -opaque-type HashMap \
  -opaque-type AutoAssertGCCallback \
  -opaque-type CompileOptions \
  -opaque-type OwningCompileOptions \
  -opaque-type ReadOnlyCompileOptions \
  -allow-unknown-types -x c++ --std=c++11 \
  -I ../target/debug/build/mozjs_sys-*/out/dist/include \
  wrapper.h \
  -DRUST_BINDGEN=1 \
  -o ../out.rs \
  -match wrapper.h \
  -match jsapi.h \
  -match jsfriendapi.h \
  -match jsalloc.h \
  -match jsbytecode.h \
  -match jspubtd.h \
  -match AllocPolicy.h \
  -match CallArgs.h \
  -match CallNonGenericMethod.h \
  -match CharacterEncoding.h \
  -match Class.h \
  -match Conversions.h \
  -match Date.h \
  -match Debug.h \
  -match EnumeratedArray.h \
  -match GCAPI.h \
  -match GCAnnotations.h \
  -match GCPolicyAPI.h \
  -match GCVariant.h \
  -match GCVector.h \
  -match HashTable.h \
  -match HeapAPI.h \
  -match Id.h \
  -match Initialization.h \
  -match LinkedList.h \
  -match LegacyIntTypes.h \
  -match MemoryMetrics.h \
  -match MemoryReporting.h \
  -match Opaque.h \
  -match Principals.h \
  -match ProfilingFrameIterator.h \
  -match ProfilingStack.h \
  -match Promise.h \
  -match Proxy.h \
  -match Range.h \
  -match RangedPtr.h \
  -match RequiredDefines.h \
  -match RootingAPI.h \
  -match SliceBudget.h \
  -match StructuredClone.h \
  -match TraceKind.h \
  -match TracingAPI.h \
  -match TrackedOptimizationInfo.h \
  -match TypeDecls.h \
  -match UbiNode.h \
  -match UbiNodeBreadthFirst.h \
  -match UbiNodeCensus.h \
  -match UbiNodeDominatorTree.h \
  -match UbiNodePostOrder.h \
  -match UbiNodeShortestPaths.h \
  -match Value.h \
  -match Vector.h \
  -match WeakMapPtr.h
