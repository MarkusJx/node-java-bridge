#include "node_classes/conversion_helper.hpp"
#include "node_classes/java_class_proxy.hpp"
#include "node_classes/java_instance_proxy.hpp"
#include "node_classes/java.hpp"

#include <napi_tools.hpp>
#include <logger.hpp>
#include <node_classes/jvm_container.hpp>

using namespace node_classes;
using namespace markusjx::logging;

void java_class_proxy::init(Napi::Env env, Napi::Object &exports) {
    Napi::Function func = DefineClass(env, "java_class_proxy", {
            InstanceMethod("getClassConstructor", &java_class_proxy::getClassConstructor, napi_enumerable),
    });

    constructor = new Napi::FunctionReference();

    *constructor = Napi::Persistent(func);
    exports.Set("java_class_proxy", func);

    env.SetInstanceData<Napi::FunctionReference>(constructor);
}

Napi::Object java_class_proxy::createInstance(const Napi::String &classname) {
    return constructor->New({classname});
}

Napi::Value java_class_proxy::getClassConstructor(const Napi::CallbackInfo &info) {
    return java_instance_proxy::getConstructor(info.Env(), Value());
}

java_class_proxy::java_class_proxy(const Napi::CallbackInfo &info) : ObjectWrap(info), clazz(nullptr), mtx() {
    CHECK_ARGS(napi_tools::string);
    TRY
        std::unique_lock<std::mutex> lock(mtx);

        // The name of the class to fetch
        classname = info[0].ToString().Utf8Value();
        StaticLogger::debugStream << "Creating a new class proxy instance for class " << classname;

        // Get our very own java environment pointer
        jni::jni_wrapper jvm = node_classes::jvm_container::attachJvm();
        clazz = std::make_shared<jni::java_class>(jvm.getClass(classname));

        Value().DefineProperty(Napi::PropertyDescriptor::Value("class.name", Napi::String::New(info.Env(), classname),
                                                               napi_enumerable));
    CATCH_EXCEPTIONS
}

Napi::FunctionReference *java_class_proxy::constructor = nullptr;