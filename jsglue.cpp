//XXXjdm whyyyyyyyyyyy
#define UINT32_MAX ((uint32_t)-1)

#include "jsapi.h"
#include "jsfriendapi.h"

enum StubType {
    PROPERTY_STUB,
    STRICT_PROPERTY_STUB,
    ENUMERATE_STUB,
    CONVERT_STUB,
    RESOLVE_STUB,
};

extern "C" {

void*
GetJSClassHookStubPointer(enum StubType type)
{
    switch (type) {
    case PROPERTY_STUB:
        return (void*)JS_PropertyStub;
    case STRICT_PROPERTY_STUB:
        return (void*)JS_StrictPropertyStub;
    case ENUMERATE_STUB:
        return (void*)JS_EnumerateStub;
    case CONVERT_STUB:
        return (void*)JS_ConvertStub;
    case RESOLVE_STUB:
        return (void*)JS_ResolveStub;
    }
    return NULL;
}

JSBool
RUST_JSVAL_IS_NULL(jsval v)
{
    return JSVAL_IS_NULL(v);
}

JSBool
RUST_JSVAL_IS_VOID(jsval v)
{
    return JSVAL_IS_VOID(v);
}

JSBool
RUST_JSVAL_IS_INT(jsval v)
{
    return JSVAL_IS_INT(v);
}

int32_t
RUST_JSVAL_TO_INT(jsval v)
{
    return JSVAL_TO_INT(v);
}

jsval
RUST_INT_TO_JSVAL(int32_t v)
{
    return INT_TO_JSVAL(v);
}

JSBool
RUST_JSVAL_IS_DOUBLE(jsval v)
{
    return JSVAL_IS_DOUBLE(v);
}

double
RUST_JSVAL_TO_DOUBLE(jsval v)
{
    return JSVAL_TO_DOUBLE(v);
}

jsval
RUST_DOUBLE_TO_JSVAL(double v)
{
    return DOUBLE_TO_JSVAL(v);
}

jsval
RUST_UINT_TO_JSVAL(uint32_t v)
{
    return UINT_TO_JSVAL(v);
}

JSBool
RUST_JSVAL_IS_NUMBER(jsval v)
{
    return JSVAL_IS_NUMBER(v);
}

JSBool
RUST_JSVAL_IS_STRING(jsval v)
{
    return JSVAL_IS_STRING(v);
}

JSString *
RUST_JSVAL_TO_STRING(jsval v)
{
    return JSVAL_TO_STRING(v);
}

jsval
RUST_STRING_TO_JSVAL(JSString *v)
{
    return STRING_TO_JSVAL(v);
}

JSBool
RUST_JSVAL_IS_OBJECT(jsval v)
{
    return !JSVAL_IS_PRIMITIVE(v) || JSVAL_IS_NULL(v);
}

JSObject *
RUST_JSVAL_TO_OBJECT(jsval v)
{
    return JSVAL_TO_OBJECT(v);
}

jsval
RUST_OBJECT_TO_JSVAL(JSObject *v)
{
    return OBJECT_TO_JSVAL(v);
}

JSBool
RUST_JSVAL_IS_BOOLEAN(jsval v)
{
    return JSVAL_IS_BOOLEAN(v);
}

JSBool
RUST_JSVAL_TO_BOOLEAN(jsval v)
{
    return JSVAL_TO_BOOLEAN(v);
}

jsval
RUST_BOOLEAN_TO_JSVAL(JSBool v)
{
    return BOOLEAN_TO_JSVAL(v);
}

JSBool
RUST_JSVAL_IS_PRIMITIVE(jsval v)
{
    return JSVAL_IS_PRIMITIVE(v);
}

JSBool
RUST_JSVAL_IS_GCTHING(jsval v)
{
    return JSVAL_IS_GCTHING(v);
}

void *
RUST_JSVAL_TO_GCTHING(jsval v)
{
    return JSVAL_TO_GCTHING(v);
}

jsval
RUST_PRIVATE_TO_JSVAL(void *v)
{
    return PRIVATE_TO_JSVAL(v);
}

void *
RUST_JSVAL_TO_PRIVATE(jsval v)
{
    return JSVAL_TO_PRIVATE(v);
}

jsval
RUST_JS_NumberValue(double d)
{
    return JS_NumberValue(d);
}

const JSJitInfo*
RUST_FUNCTION_VALUE_TO_JITINFO(jsval* v)
{
    return FUNCTION_VALUE_TO_JITINFO(*v);
}

JSBool
CallJitPropertyOp(JSJitInfo *info, JSContext* cx, JSObject* thisObj, void *specializedThis, jsval *vp)
{
    JSHandleObject* tmp = (JSHandleObject*)&thisObj;
    //XXXjdm sort out how we can do the handle thing here.
    return ((JSJitPropertyOp)info->op)(cx, *(JSHandleObject*)&tmp, specializedThis, vp);
}

void
SetFunctionNativeReserved(JSObject* fun, size_t which, js::Value* val)
{
    js::SetFunctionNativeReserved(fun, which, *val);
}

const js::Value*
GetFunctionNativeReserved(JSObject* fun, size_t which)
{
    return &js::GetFunctionNativeReserved(fun, which);
}

} // extern "C"
