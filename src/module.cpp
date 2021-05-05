#include <napi.h>
#include <jni.h>
#include <napi_tools.hpp>
#include <node_classes/node_jobject_wrapper.hpp>
#include <logger.hpp>

#include "node_classes/java.hpp"
#include "node_classes/java_class_proxy.hpp"

using namespace markusjx::logging;

void setLoggerMode(const Napi::CallbackInfo &info) {
    CHECK_ARGS(napi_tools::number);
    LogLevel level;

    switch (info[0].ToNumber().Int32Value()) {
        case 0:
            level = DEBUG;
            break;
        case 1:
            level = WARNING;
            break;
        case 2:
            level = ERROR;
            break;
        case 3:
            level = NONE;
            break;
        default:
            level = DEBUG;
    }

    StaticLogger::create(LoggerMode::MODE_CONSOLE, level, SyncMode::SYNC);
}

Napi::Object InitAll(Napi::Env env, Napi::Object exports) {
    StaticLogger::create(LoggerMode::MODE_CONSOLE, LogLevel::DEBUG, SyncMode::SYNC);
    node_classes::java::init(env, exports);
    node_classes::java_class_proxy::init(env, exports);
    node_classes::node_jobject_wrapper::init(env, exports);
    EXPORT_FUNCTION(exports, env, setLoggerMode);

    StaticLogger::debug("InitAll() called");

    return exports;
}

NODE_API_MODULE(node_java_bridge, InitAll)