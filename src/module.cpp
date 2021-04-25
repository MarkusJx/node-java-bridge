#include <napi.h>
#include <iostream>
#include <jni.h>
#include <napi_tools.hpp>
#include <windows.h>

#include "jvm_lib/shared_library.hpp"
#include "jvm_lib/jni_wrapper.hpp"
#include "node_classes/java.hpp"
#include "node_classes/java_class_proxy.hpp"
#include "node_classes/java_instance_proxy.hpp"
#include "util.hpp"

void addToClasspath(const Napi::CallbackInfo &info) {
    CHECK_ARGS(napi_tools::string);

    TRY
        const std::string path = info[0].ToString();
        node_classes::java::classpathElements.push_back(path);
    CATCH_EXCEPTIONS
}

Napi::Object InitAll(Napi::Env env, Napi::Object exports) {
    EXPORT_FUNCTION(exports, env, addToClasspath);
    node_classes::java::init(env, exports);
    node_classes::java_class_proxy::init(env, exports);

    return exports;
}

NODE_API_MODULE(node_java_bridge, InitAll)