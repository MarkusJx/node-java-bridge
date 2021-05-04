#include <jni.h>
#include <napi_tools.hpp>
#include <logger.hpp>

#include "node_classes/java_class_proxy.hpp"
#include "node_classes/jvm_container.hpp"
#include "util/util.hpp"
#include "node_classes/java.hpp"

#ifdef JNI_VERSION_1_8
#   define JNI_DEFAULT_VERSION JNI_VERSION_1_8
#else
#   define JNI_DEFAULT_VERSION JNI_VERSION_1_6
#endif

using namespace node_classes;
using namespace markusjx::logging;

void java::init(Napi::Env env, Napi::Object &exports) {
    Napi::Function func = DefineClass(env, "java", {
            StaticMethod("getClass", &java::getClass, napi_enumerable),
            InstanceMethod("appendToClasspath", &java::appendToClasspath, napi_enumerable),
            InstanceMethod("appendToClasspathAsync", &java::appendToClasspathAsync, napi_enumerable),
            InstanceAccessor("loadedJars", &java::getLoadedJars, nullptr, napi_enumerable)
    });

    auto constructor = new Napi::FunctionReference();

    *constructor = Napi::Persistent(func);
    exports.Set("java", func);

    env.SetInstanceData<Napi::FunctionReference>(constructor);
}

java::java(const Napi::CallbackInfo &info) : ObjectWrap(info), loaded_jars() {
    CHECK_ARGS(napi_tools::string, napi_tools::null | napi_tools::string);

    StaticLogger::debug("Creating a new java instance");

    TRY
        const std::string lib_path = info[0].ToString();
        jint version;

        // Parse the version string (if exists)
        if (info[1].IsString()) {
            version = util::string_to_java_version(info[1].ToString().Utf8Value());
        } else {
            version = JNI_DEFAULT_VERSION;
        }

        StaticLogger::debugStream << "Creating a java instance with version " << info[1].ToString().Utf8Value();
        jvm_container::createInstance(lib_path, version);

        Value().DefineProperty(Napi::PropertyDescriptor::Value("version",
                                                               Napi::String::New(info.Env(),
                                                                                 util::get_java_version_from_jint(
                                                                                         jvm_container::getJvm()->GetVersion())),
                                                               napi_enumerable));

        Value().DefineProperty(Napi::PropertyDescriptor::Value("wantedVersion",
                                                               Napi::String::New(info.Env(),
                                                                                 util::get_java_version_from_jint(
                                                                                         version)),
                                                               napi_enumerable));
    CATCH_EXCEPTIONS
}

Napi::Value java::getClass(const Napi::CallbackInfo &info) {
    CHECK_ARGS(napi_tools::string);
    TRY
        return java_class_proxy::createInstance(info[0].ToString());
    CATCH_EXCEPTIONS
}

Napi::Object java::getClass(const Napi::Env &env, const std::string &classname) {
    return java_class_proxy::createInstance(Napi::String::New(env, classname));
}

void java::appendToClasspath(const Napi::CallbackInfo &info) {
    CHECK_ARGS(napi_tools::string);
    TRY
        const std::string toAppend = info[0].ToString().Utf8Value();
        loaded_jars.push_back(toAppend);
        StaticLogger::debugStream << "Appending to classpath: " << toAppend;
        jvm_container::getJvm().appendClasspath(toAppend);
    CATCH_EXCEPTIONS
}

Napi::Value java::appendToClasspathAsync(const Napi::CallbackInfo &info) {
    CHECK_ARGS(napi_tools::string);
    const std::string toAppend = info[0].ToString().Utf8Value();

    return napi_tools::promises::promise<void>(info.Env(), [this, toAppend] {
        loaded_jars.push_back(toAppend);
        StaticLogger::debugStream << "Appending to classpath: " << toAppend;
        jvm_container::getJvm().attachEnv().appendClasspath(toAppend);
    });
}

Napi::Value java::getLoadedJars(const Napi::CallbackInfo &info) {
    Napi::Array res = Napi::Array::New(info.Env());
    for (size_t i = 0; i < loaded_jars.size(); i++) {
        res.Set(i, Napi::String::New(info.Env(), loaded_jars[i]));
    }

    return res;
}

java::~java() {
    // Destroy the static jvm_container instance.
    // Therefore, the jvm_container instance is
    // alive as long as the java object is alive.
    // The java object is alive as long as it exists
    // in the index.js file, so it ceases to exist
    // either when
    // A, the module is unloaded or
    // B, java.createInstance is called
    jvm_container::destroyInstance();
}
