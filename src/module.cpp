#include <napi.h>
#include <jni.h>
#include <napi_tools.hpp>

#ifdef ENABLE_LOGGING
#   include <logger.hpp>
#endif //ENABLE_LOGGING

#include "node_classes/java.hpp"
#include "node_classes/java_class_proxy.hpp"
#include "node_classes/java_function_caller.hpp"
#include "node_classes/node_jobject_wrapper.hpp"
#include "node_classes/stdout_redirect.hpp"
#include "node_classes/jvm_container.hpp"

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
    CHECK_ARGS(napi_tools::string, napi_tools::string);
    TRY
        node_classes::java::set_root_dir(info[1].ToString().Utf8Value());
        node_classes::java::set_native_lib_path(info[0].ToString().Utf8Value());
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
    node_classes::stdout_redirect::init(env, exports);

    EXPORT_FUNCTION(exports, env, setLoggerMode);
    EXPORT_FUNCTION(exports, env, setNativeLibraryPath);

#ifdef ENABLE_LOGGING
    StaticLogger::debug("InitAll() called");
#endif //ENABLE_LOGGING

    std::atexit([] {
#ifdef ENABLE_LOGGING
        StaticLogger::debug("Running exit action");
#endif //ENABLE_LOGGING
        try {
            node_classes::jvm_container::destroyInstance();
#ifdef ENABLE_LOGGING
        } catch (const std::exception &e) {
            StaticLogger::errorStream << "The cleanup action threw an exception: " << e.what();
#endif //ENABLE_LOGGING
        } catch (...) {}
#ifdef ENABLE_LOGGING
        StaticLogger::debug("Exit action complete");
#endif //ENABLE_LOGGING
    });

    return exports;
}

NODE_API_MODULE(node_java_bridge, InitAll)