#include <napi_tools.hpp>

#include "node_classes/conversion_helper.hpp"
#include "node_classes/java_instance_proxy.hpp"
#include "node_classes/java_class_proxy.hpp"
#include "node_classes/node_jobject_wrapper.hpp"

#include <logger.hpp>

#define GET_JAVA_INSTANCE() info.This().ToObject().Get("java.instance").ToObject()

using namespace node_classes;
using namespace markusjx::logging;

Napi::Value java_instance_proxy::staticGetter(const Napi::CallbackInfo &info) {
    Napi::Object class_proxy_instance = info.This().ToObject().Get("class.proxy.instance").ToObject();
    auto *ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy_instance);
    std::string toRetrieve(static_cast<const char *>(info.Data()));
    std::unique_lock lock(ptr->mtx);
    jni::java_field field = ptr->clazz->static_fields.at(toRetrieve);

    Napi::Object java_instance = class_proxy_instance.Get("java.instance").ToObject();
    return conversion_helper::static_java_field_to_object(field, ptr->clazz->clazz, info.Env(), java_instance);
}

void java_instance_proxy::staticSetter(const Napi::CallbackInfo &info, const Napi::Value &value) {
    Napi::Object class_proxy_instance = info.This().ToObject().Get("class.proxy.instance").ToObject();
    auto *ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy_instance);
    std::string toRetrieve(static_cast<const char *>(info.Data()));

    jni::java_field field = ptr->clazz->static_fields.at(toRetrieve);
    field.setStatic(ptr->clazz->clazz,
                    conversion_helper::value_to_jobject(info.Env(), ptr->jvm.attachEnv(), value, field.signature));
}

Napi::Value java_instance_proxy::callStaticFunction(const Napi::CallbackInfo &info) {
    const std::string functionName(static_cast<const char *>(info.Data()));
    StaticLogger::debugStream << "Calling static method '" << functionName << "' with " << info.Length()
                              << " argument(s)";
    Napi::Object class_proxy_instance = info.This().ToObject().Get("class.proxy.instance").ToObject();
    auto *ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy_instance);

    Napi::Object java_instance = class_proxy_instance.Get("java.instance").ToObject();

    TRY
        return conversion_helper::call_matching_static_function(info, java_instance, ptr->clazz->clazz,
                                                                ptr->clazz->static_functions.at(functionName));
    CATCH_EXCEPTIONS
}

Napi::Function java_instance_proxy::getConstructor(Napi::Env env, const Napi::Object &class_proxy) {
    std::vector<Napi::ObjectWrap<java_instance_proxy>::PropertyDescriptor> properties;
    java_class_proxy *cls = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy);

    StaticLogger::debugStream << "Creating a constructor for java class '" << cls->classname << "'";

    properties.push_back(StaticValue("class.proxy.instance", class_proxy, napi_enumerable));

    StaticLogger::debugStream << "Setting getters and setters for " << cls->clazz->static_fields.size()
                              << " static fields";
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

    StaticLogger::debugStream << "Setting functions for " << cls->clazz->static_functions.size()
                              << " static functions";
    for (const auto &f : cls->clazz->static_functions) {
        properties.push_back(StaticMethod(f.first.c_str(), &java_instance_proxy::callStaticFunction, napi_enumerable,
                                          (void *) f.first.c_str()));
    }

    return DefineClass(env, "java_instance_proxy", properties);
}

Napi::Value java_instance_proxy::fromJObject(Napi::Env env, const jni::jobject_wrapper<jobject> &obj,
                                             const Napi::Object &class_proxy) {
    StaticLogger::debug("Creating a class instance proxy from an existing jobject");

    Napi::Object jobject_wrapper = node_jobject_wrapper::createInstance();
    Napi::ObjectWrap<node_jobject_wrapper>::Unwrap(jobject_wrapper)->setData(obj);

    Napi::Function constructor = getConstructor(env, class_proxy);
    return constructor.New({jobject_wrapper});
}

java_instance_proxy::java_instance_proxy(const Napi::CallbackInfo &info) : ObjectWrap(info) {
    Napi::Object class_proxy_instance = info.NewTarget().ToObject().Get("class.proxy.instance").ToObject();
    java_class_proxy *class_ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy_instance);
    Napi::Object java_instance = class_proxy_instance.Get("java.instance").ToObject();

    StaticLogger::debugStream << "Creating a new '" << class_ptr->classname << "' instance";

    {
        std::unique_lock lock(class_ptr->mtx);
        clazz = class_ptr->clazz;
        jvm = class_ptr->jvm;
    }

    StaticLogger::debugStream << "Setting getters and setters for " << clazz->fields.size() << " fields";
    for (const auto &f : clazz->fields) {
        const auto getter = [f, this](const Napi::CallbackInfo &info) -> Napi::Value {
            TRY
                return conversion_helper::jobject_to_value(info.Env(), GET_JAVA_INSTANCE(), f.second.get(object),
                                                           f.second.signature);
            CATCH_EXCEPTIONS
        };

        if (f.second.isFinal) {
            Value().DefineProperty(Napi::PropertyDescriptor::Accessor(f.first, getter, napi_enumerable));
        } else {
            const auto setter = [f, this](const Napi::CallbackInfo &info) -> void {
                if (info.Length() != 1) {
                    throw Napi::TypeError::New(info.Env(),
                                               "Tried accessing a setter with more or less than one argument");
                }

                TRY
                    f.second.set(object,
                                 conversion_helper::value_to_jobject(info.Env(), jvm, info[0], f.second.signature));
                CATCH_EXCEPTIONS
            };

            Value().DefineProperty(Napi::PropertyDescriptor::Accessor(f.first, getter, setter, napi_enumerable));
        }
    }

    StaticLogger::debugStream << "Setting functions for " << clazz->functions.size() << " java functions";
    for (const auto &f : clazz->functions) {
        const auto function = [f, this](const Napi::CallbackInfo &info) -> Napi::Value {
            StaticLogger::debugStream << "Calling instance method '" << f.first << "' with " << info.Length()
                                      << " argument(s)";
            TRY
                return conversion_helper::call_matching_function(info, GET_JAVA_INSTANCE(), object, f.second);
            CATCH_EXCEPTIONS
        };

        Value().DefineProperty(Napi::PropertyDescriptor::Function(f.first, function, napi_enumerable));
    }

    Value().DefineProperty(Napi::PropertyDescriptor::Value("java.instance", java_instance, napi_enumerable));

    if (info.Length() == 1 && node_jobject_wrapper::instanceOf(info[0].ToObject())) {
        StaticLogger::debug("The class constructor was called with a node_jobject_wrapper as first argument, "
                            "creating the class using that information");
        object = Napi::ObjectWrap<node_jobject_wrapper>::Unwrap(info[0].ToObject())->getObject();
    } else {
        StaticLogger::debugStream << "Trying to find a matching constructor for the " << info.Length()
                                  << " provided arguments";
        object = conversion_helper::match_constructor_arguments(info, jvm.attachEnv(), clazz->constructors);
    }
}
