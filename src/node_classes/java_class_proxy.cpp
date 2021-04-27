#include "node_classes/conversion_helper.hpp"
#include "node_classes/java_class_proxy.hpp"
#include "node_classes/java_instance_proxy.hpp"
#include "node_classes/java.hpp"

#include <napi_tools.hpp>
#include <logger.hpp>

using namespace node_classes;
using namespace logger;

void java_class_proxy::init(Napi::Env env, Napi::Object &exports) {
    Napi::Function func = DefineClass(env, "java_class_proxy", {
            InstanceMethod<&java_class_proxy::getClassConstructor>("getClassConstructor")
    });

    constructor = new Napi::FunctionReference();

    *constructor = Napi::Persistent(func);
    exports.Set("java_class_proxy", func);

    env.SetInstanceData<Napi::FunctionReference>(constructor);
}

Napi::Object java_class_proxy::createInstance(const Napi::Object &java_instance, const Napi::String &classname) {
    return constructor->New({java_instance, classname});
}

Napi::Value java_class_proxy::getClassConstructor(const Napi::CallbackInfo &info) {
    return java_instance_proxy::getConstructor(info.Env(), Value());
}

java_class_proxy::java_class_proxy(const Napi::CallbackInfo &info) : ObjectWrap(info), clazz(nullptr), mtx() {
    CHECK_ARGS(napi_tools::object, napi_tools::string);
    std::unique_lock<std::mutex> lock(mtx);

    // This holds the actual (calling) java class (java.hpp) instance
    Napi::Object java_instance = info[0].ToObject();

    // The name of the class to fetch
    classname = info[1].ToString().Utf8Value();
    StaticLogger::debugStream << "Creating a new class proxy instance for class " << classname;

    // The java class instance pointer
    java *java_ptr = Napi::ObjectWrap<java>::Unwrap(java_instance);

    // Get our very own java environment pointer
    jvm = java_ptr->java_environment;
    clazz = std::make_shared<jni::java_class>(jvm.getClass(classname));

    Value().DefineProperty(Napi::PropertyDescriptor::Value("class.name", Napi::String::New(info.Env(), classname),
                                                           napi_enumerable));
    Value().DefineProperty(Napi::PropertyDescriptor::Value("java.instance", java_instance, napi_enumerable));
}

Napi::FunctionReference *java_class_proxy::constructor = nullptr;