#ifndef NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP
#define NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP

#include <napi.h>

#include "jvm_lib/jni_wrapper.hpp"

namespace conversion_helper {
    Napi::Value static_java_field_to_object(const jni::java_field &to_convert, jclass clazz, const Napi::Env &env,
                                            const Napi::Object &java_instance);

    Napi::Value jobject_to_value(const Napi::Env &env, const Napi::Object &java_instance,
                                 const jni::jobject_wrapper<jobject> &obj, std::string signature);

    jni::jobject_wrapper<jobject> value_to_jobject(const Napi::Env &env, const jni::jni_wrapper &j_env,
                                                   const Napi::Value &value, std::string signature);

    std::string napi_valuetype_to_string(napi_valuetype type);

    jni::jobject_wrapper<jobject> match_constructor_arguments(const Napi::CallbackInfo &args,
                                                              const jni::jni_wrapper &j_env,
                                                              const std::vector<jni::java_constructor> &constructors);

    const jni::java_constructor *find_matching_constructor(const Napi::CallbackInfo &args,
                                                           const jni::jni_wrapper &j_env,
                                                           const std::vector<jni::java_constructor> &constructors,
                                                           std::vector<jni::jobject_wrapper<jobject>> &outArgs,
                                                           std::string &error);

    Napi::Value call_matching_function(const Napi::CallbackInfo &args, const Napi::Object &java_instance,
                                       const jni::jobject_wrapper<jobject> &classInstance,
                                       const std::vector<jni::java_function> &functions);

    Napi::Value call_matching_static_function(const Napi::CallbackInfo &info, const Napi::Object &java_instance,
                                              jclass clazz, const std::vector<jni::java_function> &functions);
}

#endif //NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP
