#include <napi_tools.hpp>

#include "node_classes/conversion_helper.hpp"
#include "node_classes/java_instance_proxy.hpp"
#include "node_classes/java_class_proxy.hpp"

using namespace node_classes;

Napi::Value java_instance_proxy::staticGetter(const Napi::CallbackInfo &info) {
    std::cout << "Getter invoked" << std::endl;
    auto *ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(
            info.This().ToObject().Get("class_proxy_instance").ToObject());
    std::string toRetrieve(static_cast<const char *>(info.Data()));
    std::unique_lock lock(ptr->mtx);
    jni::java_field field = ptr->clazz->static_fields.at(toRetrieve);

    return conversion_helper::static_java_field_to_object(field, ptr->clazz->clazz, info.Env(), ptr->jvm);
}

void java_instance_proxy::staticSetter(const Napi::CallbackInfo &info, const Napi::Value &value) {

}

Napi::Value java_instance_proxy::callStaticFunction(const Napi::CallbackInfo &info) {
    return info.Env().Null();
}

Napi::Function java_instance_proxy::getConstructor(Napi::Env env, const Napi::Object &class_proxy) {
    std::vector<Napi::ObjectWrap<java_instance_proxy>::PropertyDescriptor> properties;
    java_class_proxy *cls = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy);

    properties.push_back(StaticValue("class_proxy_instance", class_proxy, napi_enumerable));
    properties.push_back(StaticValue("java_class_name", Napi::String::New(env, cls->classname), napi_enumerable));

    for (const auto &f : cls->clazz->static_fields) {
        if (f.second.isFinal) {
            properties.push_back(StaticAccessor(f.first.c_str(), &java_instance_proxy::staticGetter, nullptr,
                                                napi_enumerable, (void *) f.first.c_str()));
        } else {
            properties.push_back(StaticAccessor(f.first.c_str(), &java_instance_proxy::staticGetter,
                                                &java_instance_proxy::staticSetter, napi_enumerable,
                                                (void *) f.first.c_str()));
        }
    }

    for (const auto &f : cls->clazz->static_functions) {
        properties.push_back(StaticMethod(f.first.c_str(), &java_instance_proxy::callStaticFunction, napi_enumerable,
                                          (void *) f.first.c_str()));
    }

    return DefineClass(env, "java_instance_proxy", properties);
}

java_instance_proxy::java_instance_proxy(const Napi::CallbackInfo &info) : ObjectWrap(info) {
    Napi::Object class_proxy_instance = info.NewTarget().ToObject().Get("class_proxy_instance").ToObject();
    java_class_proxy *class_ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy_instance);

    {
        std::unique_lock lock(class_ptr->mtx);
        clazz = class_ptr->clazz;
        jvm = class_ptr->jvm;
    }

    for (const auto &f : clazz->fields) {
        const auto getter = [](const Napi::CallbackInfo &info) -> Napi::Value {
            return info.Env().Null();
        };

        if (f.second.isFinal) {
            Value().DefineProperty(Napi::PropertyDescriptor::Accessor(f.first, getter, napi_enumerable));
        } else {
            const auto setter = [](const Napi::CallbackInfo &info) -> void {

            };

            Value().DefineProperty(Napi::PropertyDescriptor::Accessor(f.first, getter, setter, napi_enumerable));
        }
    }

    for (const auto &f : clazz->functions) {
        const auto function = [](const Napi::CallbackInfo &info) -> Napi::Value {
            return info.Env().Null();
        };

        Value().DefineProperty(Napi::PropertyDescriptor::Function(f.first, function, napi_enumerable));
    }
}
