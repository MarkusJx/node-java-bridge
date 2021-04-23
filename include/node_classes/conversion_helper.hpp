#ifndef NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP
#define NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP

#include <napi.h>

#include "jvm_lib/jni_wrapper.hpp"

namespace conversion_helper {
    Napi::Value static_java_field_to_object(const jni::java_field &to_convert, jclass clazz, const Napi::Env &env,
                                            const jni::jni_wrapper &j_env);
}

#endif //NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP
