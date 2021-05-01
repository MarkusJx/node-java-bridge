#include <napi.h>
#include <jni.h>
#include <napi_tools.hpp>
#include <node_classes/node_jobject_wrapper.hpp>
#include <logger.hpp>

#include "node_classes/java.hpp"
#include "node_classes/java_class_proxy.hpp"

using namespace markusjx::logging;

Napi::Object InitAll(Napi::Env env, Napi::Object exports) {
    StaticLogger::create(LoggerMode::MODE_CONSOLE, LogLevel::DEBUG, SyncMode::SYNC);
    node_classes::java::init(env, exports);
    node_classes::java_class_proxy::init(env, exports);
    node_classes::node_jobject_wrapper::init(env, exports);

    StaticLogger::debug("InitAll() called");

    return exports;
}

NODE_API_MODULE(node_java_bridge, InitAll)