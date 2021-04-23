#include <jni.h>
#include <napi_tools.hpp>
#include "node_classes/java.hpp"
#include "node_classes/java_class_proxy.hpp"
#include "util.hpp"

#ifdef JNI_VERSION_1_8
#   define JNI_DEFAULT_VERSION JNI_VERSION_1_8
#else
#   define JNI_DEFAULT_VERSION JNI_VERSION_1_6
#endif

using namespace node_classes;

Napi::Object java::init(Napi::Env env, Napi::Object exports) {
    Napi::Function func = DefineClass(env, "java", {
        InstanceMethod<&java::getClass>("getClass"),
    });

    auto constructor = new Napi::FunctionReference();

    *constructor = Napi::Persistent(func);
    exports.Set("java", func);

    env.SetInstanceData<Napi::FunctionReference>(constructor);
    return exports;
}

java::java(const Napi::CallbackInfo &info) : ObjectWrap(info) {
    CHECK_ARGS(napi_tools::string, napi_tools::undefined | napi_tools::string);

    TRY
        const std::string lib_path = info[0].ToString();
        jint version;
        if (info[1].IsString()) {
            version = util::string_to_java_version(info[1].ToString().Utf8Value());
        } else {
            version = JNI_DEFAULT_VERSION;
        }

        const std::string cp = util::classpath_elements_to_classpath(classpathElements);
        java_environment = jni::jvm_wrapper(lib_path, version, cp);
    CATCH_EXCEPTIONS
}

Napi::Value java::getClass(const Napi::CallbackInfo &info) {
    CHECK_ARGS(napi_tools::string);

    TRY
        return java_class_proxy::createInstance(info.Env(), Value(), info[0].ToString());
    CATCH_EXCEPTIONS
}

java::~java() = default;

std::vector<std::string> java::classpathElements;
