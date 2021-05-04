#ifndef NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP
#define NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP

#include <napi.h>

#include "jvm_lib/jni_wrapper.hpp"

namespace conversion_helper {
    Napi::Value static_java_field_to_object(const jni::java_field &to_convert, jclass clazz, const Napi::Env &env);

    Napi::Value jobject_to_value(const Napi::Env &env, const jni::jobject_wrapper<jobject> &obj, std::string signature);

    jni::jobject_wrapper<jobject> value_to_jobject(const Napi::Env &env, const Napi::Value &value,
                                                   std::string signature);

    std::string napi_valuetype_to_string(napi_valuetype type);

    jni::jobject_wrapper<jobject> match_constructor_arguments(const Napi::CallbackInfo &args,
                                                              const std::vector<jni::java_constructor> &constructors);

    const jni::java_constructor *find_matching_constructor(const Napi::CallbackInfo &args,
                                                           const std::vector<jni::java_constructor> &constructors,
                                                           std::vector<jni::jobject_wrapper<jobject>> &outArgs,
                                                           std::string &error);

    Napi::Value call_matching_function(const Napi::CallbackInfo &args,
                                       const jni::jobject_wrapper<jobject> &classInstance,
                                       const std::vector<jni::java_function> &functions);

    Napi::Value call_matching_static_function(const Napi::CallbackInfo &info, jclass clazz,
                                              const std::vector<jni::java_function> &functions);


    const jni::java_function *find_matching_function(const Napi::CallbackInfo &args,
                                                     const std::vector<jni::java_function> &functions,
                                                     std::vector<jni::jobject_wrapper<jobject>> &outArgs,
                                                     std::string &error,
                                                     std::vector<jvalue> &outValues);

    jvalue call_function(const jni::java_function &function, const jni::jobject_wrapper<jobject> &classInstance,
                         const std::vector<jvalue> &args);

    jvalue call_static_function(const jni::java_function &function, jclass clazz, const std::vector<jvalue> &args);

    Napi::Value jvalue_to_napi_value(jvalue value, const std::string &signature, const Napi::Env &env);
}

#endif //NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP
