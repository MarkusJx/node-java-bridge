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

void java::init(Napi::Env env, Napi::Object &exports) {
    Napi::Function func = DefineClass(env, "java", {});

    auto constructor = new Napi::FunctionReference();

    *constructor = Napi::Persistent(func);
    exports.Set("java", func);

    env.SetInstanceData<Napi::FunctionReference>(constructor);
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

        Value().DefineProperty(
                Napi::PropertyDescriptor::Value("classpath", Napi::String::New(info.Env(), cp), napi_enumerable));
        Value().DefineProperty(Napi::PropertyDescriptor::Value("version", Napi::Number::New(info.Env(),
                                                                                            java_environment->GetVersion()),
                                                               napi_enumerable));
    CATCH_EXCEPTIONS
}

java::~java() = default;

std::vector<std::string> java::classpathElements;
