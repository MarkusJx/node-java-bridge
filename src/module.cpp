#include <napi.h>
#include <jni.h>
#include <napi_tools.hpp>
#include <node_classes/node_jobject_wrapper.hpp>

#ifdef ENABLE_LOGGING
#   include <logger.hpp>
#endif //ENABLE_LOGGING

#include "node_classes/java.hpp"
#include "node_classes/java_class_proxy.hpp"
#include "node_classes/java_function_caller.hpp"

#ifdef ENABLE_LOGGING
using namespace markusjx::logging;

#   pragma message("INFO: Building with logging enabled")
#endif //ENABLE_LOGGING

#ifndef NDEBUG
#   pragma message("INFO: Building in debug mode")
#endif //NDEBUG

void setLoggerMode(const Napi::CallbackInfo &info) {
    CHECK_ARGS(napi_tools::number);
#ifdef ENABLE_LOGGING
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
#endif //ENABLE_LOGGING
}

void setNativeLibraryPath(const Napi::CallbackInfo &info) {
    CHECK_ARGS(napi_tools::string, napi_tools::buffer);
    TRY
        auto buf = info[1].As<Napi::Buffer<char>>();
        std::vector<char> data(buf.Data(), buf.Data() + buf.Length());

        node_classes::java_function_caller::setLibraryPath(info[0].ToString().Utf8Value(), data);
    CATCH_EXCEPTIONS
}

Napi::Object InitAll(Napi::Env env, Napi::Object exports) {
#ifdef ENABLE_LOGGING
    StaticLogger::create(LoggerMode::MODE_CONSOLE, LogLevel::DEBUG, SyncMode::SYNC);
#endif //ENABLE_LOGGING
    node_classes::java::init(env, exports);
    node_classes::java_class_proxy::init(env, exports);
    node_classes::node_jobject_wrapper::init(env, exports);
    node_classes::java_function_caller::init(env, exports);

    EXPORT_FUNCTION(exports, env, setLoggerMode);
    EXPORT_FUNCTION(exports, env, setNativeLibraryPath);

#ifdef ENABLE_LOGGING
    StaticLogger::debug("InitAll() called");
#endif //ENABLE_LOGGING

    return exports;
}

NODE_API_MODULE(node_java_bridge, InitAll)