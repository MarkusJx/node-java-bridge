#include "node_classes/conversion_helper.hpp"
#include "node_classes/java_class_proxy.hpp"
#include "node_classes/java.hpp"

#include <napi_tools.hpp>

using namespace node_classes;

Napi::Value staticValueGetter(const Napi::CallbackInfo &info) {
    auto *ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(info.This().ToObject());
    std::string toRetrieve(static_cast<const char *>(info.Data()));
    jni::java_field field = ptr->clazz->static_fields.at(toRetrieve);

    return conversion_helper::static_java_field_to_object(field, ptr->clazz->clazz, info.Env(), ptr->env);
}

void staticValueSetter(const Napi::CallbackInfo &info) {
    auto *ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(info.This().ToObject());
    // TODO: Convert the data
}

Napi::Object java_class_proxy::init(Napi::Env env, Napi::Object exports) {
    Napi::Function func = DefineClass(env, "java_class_proxy", {});

    auto constructor = new Napi::FunctionReference();

    *constructor = Napi::Persistent(func);
    exports.Set("java_class_proxy", func);

    env.SetInstanceData<Napi::FunctionReference>(constructor);
    return exports;
}

Napi::Object java_class_proxy::createInstance(Napi::Env env, const Napi::Object &parent,
                                              const Napi::String &classname) {
    auto *constructor = env.GetInstanceData<Napi::FunctionReference>();
    return constructor->New({parent, classname});
}

java_class_proxy::java_class_proxy(const Napi::CallbackInfo &info) : ObjectWrap(info), clazz(nullptr) {
    CHECK_ARGS(napi_tools::object, napi_tools::string);

    // This holds the actual (calling) java class (java.hpp) instance
    Napi::Object java_instance = info[0].ToObject();

    // The name of the class to fetch
    std::string classname = info[1].ToString();

    // The java class instance pointer
    java *java_ptr = Napi::ObjectWrap<java>::Unwrap(java_instance);

    // Get our very own java environment pointer
    env = java_ptr->java_environment.attachEnv();
    clazz = std::make_shared<jni::java_class>(env.getClass(classname));

    Value().Set("className", Napi::String::New(info.Env(), classname));
    for (const auto &f : clazz->static_fields) {
        Value().DefineProperty(Napi::PropertyDescriptor::Accessor<staticValueGetter, staticValueSetter>(f.first,
                                                                                                        napi_default,
                                                                                                        (void *) f.first.c_str()));
    }
}
