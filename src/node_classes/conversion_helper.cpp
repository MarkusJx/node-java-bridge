#include "node_classes/conversion_helper.hpp"

Napi::Value
conversion_helper::static_java_field_to_object(const jni::java_field &to_convert, jclass clazz, const Napi::Env &env,
                                               const jni::jni_wrapper &j_env) {
    if (to_convert.signature == "I") {
        // Value is an integer
        return Napi::Number::New(env, j_env.jobject_to_jint(to_convert.getStatic(clazz)));
    } else if (to_convert.signature == "Z") {
        // Value is a boolean
        return Napi::Boolean::New(env, j_env.jobject_to_jboolean(to_convert.getStatic(clazz)));
    } else if (to_convert.signature == "B") {
        // Value is a byte
        // TODO
        return env.Null();
    } else if (to_convert.signature == "C") {
        // Value is a char
        return Napi::String::New(env, std::to_string(j_env.jobject_to_jchar(to_convert.getStatic(clazz))));
    } else if (to_convert.signature == "S") {
        // Value is a short
        return Napi::Number::New(env, j_env.jobject_to_jshort(to_convert.getStatic(clazz)));
    } else if (to_convert.signature == "J") {
        // Value is a long
        // TODO: Return raw values
        return Napi::Number::New(env, j_env.jobject_to_jlong(to_convert.getStatic(clazz)));
    } else if (to_convert.signature == "F") {
        // Value is a float
        return Napi::Number::New(env, j_env.jobject_to_jfloat(to_convert.getStatic(clazz)));
    } else if (to_convert.signature == "D") {
        // Value is a double
        return Napi::Number::New(env, j_env.jobject_to_jdouble(to_convert.getStatic(clazz)));
    } else if (to_convert.signature == "Ljava/lang/String;") {
        // Value is a string
        return Napi::String::New(env,
                                 j_env.jstring_to_string(reinterpret_cast<jstring>(to_convert.getStatic(clazz).obj)));
    } else {
        // TODO: Classes and arrays
    }
    return env.Null();
}