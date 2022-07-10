#include <napi_tools.hpp>

#include "node_classes/conversion_helper.hpp"
#include "node_classes/java_instance_proxy.hpp"
#include "node_classes/java_class_proxy.hpp"
#include "node_classes/node_jobject_wrapper.hpp"
#include "node_classes/jvm_container.hpp"
#include "node_classes/java.hpp"
#include "definitions.hpp"

#ifdef ENABLE_LOGGING
#   include <logger.hpp>
#endif //ENABLE_LOGGING

#include <utility>

#ifdef JAVA_WINDOWS
#   define STRDUP(str) _strdup(str)
#else
#   define STRDUP(str) strdup(str)
#endif //JAVA_WINDOWS

using namespace node_classes;
#ifdef ENABLE_LOGGING
using namespace markusjx::logging;
#endif //ENABLE_LOGGING

/**
 * A helper which will create the class instance
 */
class instance_def {
public:
    instance_def() = default;

    instance_def(util::persistent_object class_proxy, jni::jobject_wrapper<jobject> object)
            : class_proxy(std::move(class_proxy)), object(std::move(object)) {}

    JAVA_UNUSED static Napi::Value toNapiValue(const Napi::Env &env, const instance_def &c) {
        return java_instance_proxy::fromJObject(env, c.object, c.class_proxy.value());
    }

private:
    util::persistent_object class_proxy;
    jni::jobject_wrapper<jobject> object;
};

/**
 * A helper which will convert the function return value
 */
class jvalue_converter {
public:
    jvalue_converter() : value(), signature() {}

    jvalue_converter(jvalue value, java_type signature) : value(value), signature(std::move(signature)) {}

    JAVA_UNUSED static Napi::Value toNapiValue(const Napi::Env &env, const jvalue_converter &c) {
        return conversion_helper::jvalue_to_napi_value(c.value, c.signature, env);
    }

private:
    java_type signature;
    jvalue value;
};

Napi::Value java_instance_proxy::staticGetter(const Napi::CallbackInfo &info) {
    Napi::Object class_proxy_instance = info.This().ToObject().Get("class.proxy.instance").ToObject();
    auto *ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy_instance);
    std::string toRetrieve(static_cast<const char *>(info.Data()));
    std::unique_lock lock(ptr->mtx);
    jni::java_field field = ptr->clazz->static_fields.at(toRetrieve);

    jni::jobject_wrapper<jobject> tmp;
    return conversion_helper::jvalue_to_napi_value(field.getStatic(ptr->clazz->clazz, tmp), field.signature,
                                                   info.Env());
}

void java_instance_proxy::staticSetter(const Napi::CallbackInfo &info, const Napi::Value &value) {
    Napi::Object class_proxy_instance = info.This().ToObject().Get("class.proxy.instance").ToObject();
    auto *ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy_instance);
    std::string toRetrieve(static_cast<const char *>(info.Data()));

    jni::java_field field = ptr->clazz->static_fields.at(toRetrieve);
    std::vector<jni::jobject_wrapper<jobject>> tmp;
    field.setStatic(ptr->clazz->clazz,
                    conversion_helper::napi_value_to_jvalue(info.Env(), value, field.signature, tmp, true));
}

Napi::Value java_instance_proxy::callStaticFunction(const Napi::CallbackInfo &info) {
    const std::string functionName(static_cast<const char *>(info.Data()));
#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Calling static method '" << functionName << "' with " << info.Length()
                              << " argument(s)";
#endif //ENABLE_LOGGING
    Napi::Object class_proxy_instance = info.This().ToObject().Get("class.proxy.instance").ToObject();
    auto *ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy_instance);

    TRY
        return conversion_helper::call_matching_static_function(info, ptr->clazz->clazz,
                                                                ptr->clazz->static_functions.at(functionName));
    CATCH_EXCEPTIONS
}

Napi::Value java_instance_proxy::callStaticFunctionAsync(const Napi::CallbackInfo &info) {
    const std::string functionName(static_cast<const char *>(info.Data()));
#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Calling static method '" << functionName << "' with " << info.Length()
                              << " argument(s) (async)";
#endif //ENABLE_LOGGING
    Napi::Object class_proxy_instance = info.This().ToObject().Get("class.proxy.instance").ToObject();
    auto *ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy_instance);

    std::vector<jni::jobject_wrapper<jobject>> args;
    std::string error;
    std::vector<jvalue> values;
    auto func = conversion_helper::find_matching_function(info, ptr->clazz->static_functions.at(functionName), args,
                                                          error, values);
    jclass clazz = ptr->clazz->clazz;

    return napi_tools::promises::promise<jvalue_converter>(info.Env(), [args, error, values, func, clazz] {
        if (func == nullptr) {
            throw std::runtime_error(error);
        }

        // 3.4s
        jvalue val = conversion_helper::call_static_function(*func, clazz, values);
        //std::cout << "Thread id: " << std::this_thread::get_id() << std::endl;
        // ~11s
        return jvalue_converter(val, func->returnType);
    });
}

std::vector<Napi::ObjectWrap<java_instance_proxy>::PropertyDescriptor>
java_instance_proxy::generateProperties(const Napi::Object &class_proxy, const Napi::Env &env) {
#ifdef ENABLE_LOGGING
    StaticLogger::debug("Unwrapping the class proxy");
#endif //ENABLE_LOGGING
    std::vector<Napi::ObjectWrap<java_instance_proxy>::PropertyDescriptor> properties;
    java_class_proxy *cls = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy);

#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Creating a constructor for java class '" << cls->classname << "'";
#endif //ENABLE_LOGGING

    properties.push_back(StaticValue("class.proxy.instance", class_proxy, napi_enumerable));

#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Setting getters and setters for " << cls->clazz->static_fields.size()
                              << " static fields";
#endif //ENABLE_LOGGING
    for (const auto &f: cls->clazz->static_fields) {
        if (f.second.isFinal) {
            properties.push_back(StaticAccessor(f.first.c_str(), &java_instance_proxy::staticGetter, nullptr,
                                                napi_enumerable, (void *) f.first.c_str()));
        } else {
            properties.push_back(StaticAccessor(f.first.c_str(), &java_instance_proxy::staticGetter,
                                                &java_instance_proxy::staticSetter, napi_enumerable,
                                                (void *) f.first.c_str()));
        }
    }

#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Setting functions for " << cls->clazz->static_functions.size()
                              << " static functions";
#endif //ENABLE_LOGGING
    for (const auto &f: cls->clazz->static_functions) {
        char *name = STRDUP((f.first + "Sync").c_str());
        cls->additionalData.emplace_back(name, free);
#ifdef ENABLE_LOGGING
        StaticLogger::debugStream << "Creating static method: " << name;
#endif //ENABLE_LOGGING
        properties.push_back(StaticMethod(name,
                                          &java_instance_proxy::callStaticFunction, napi_enumerable,
                                          (void *) f.first.c_str()));

#ifdef ENABLE_LOGGING
        StaticLogger::debugStream << "Creating static method: " << f.first;
#endif //ENABLE_LOGGING
        properties.push_back(StaticMethod(f.first.c_str(), &java_instance_proxy::callStaticFunctionAsync,
                                          napi_enumerable, (void *) f.first.c_str()));
    }

    if (cls->clazz->constructors.size() > 0) {
#ifdef ENABLE_LOGGING
        StaticLogger::debugStream << "Creating 'newInstance' method";
#endif //ENABLE_LOGGING
        properties.push_back(StaticMethod("newInstance", &java_instance_proxy::newInstance, napi_enumerable));
    }

#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Creating 'instanceOf' method";
#endif //ENABLE_LOGGING
    properties.push_back(InstanceMethod("instanceOf", &java_instance_proxy::instanceOf, napi_enumerable));

#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Done creating class '" << cls->classname << "'";
#endif //ENABLE_LOGGING
    return properties;
}

Napi::Function java_instance_proxy::getConstructor(Napi::Env env, const Napi::Object &class_proxy) {
    return DefineClass(env, "java_instance_proxy", generateProperties(class_proxy, env));
}

Napi::Value java_instance_proxy::newInstance(const Napi::CallbackInfo &info) {
    std::vector<jni::jobject_wrapper<jobject>> args;

    Napi::Object class_proxy = info.This().ToObject().Get("class.proxy.instance").ToObject();
    java_class_proxy *class_ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy);

    std::string error;
    util::persistent_object class_ref(class_proxy);
    auto constructor = conversion_helper::find_matching_constructor(info, class_ptr->clazz->constructors, args, error);

    return napi_tools::promises::promise<instance_def>(info.Env(), [class_ref, constructor, args, error] {
        if (!constructor || !error.empty()) {
            throw std::runtime_error(error);
        }

        jni::jobject_wrapper<jobject> j_object = constructor->newInstance(args);
        return instance_def(class_ref, j_object);
    });
}

Napi::Value java_instance_proxy::instanceOf(const Napi::CallbackInfo &info) {
    CHECK_ARGS(napi_tools::string);

    TRY
        jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
        jclass j_clazz = j_env.getJClass(info[0].ToString().Utf8Value());
        return Napi::Boolean::New(info.Env(), j_env->IsInstanceOf(object, j_clazz));
    CATCH_EXCEPTIONS
}

Napi::Value java_instance_proxy::fromJObject(Napi::Env env, const jni::jobject_wrapper<jobject> &obj,
                                             const Napi::Object &class_proxy) {
#ifdef ENABLE_LOGGING
    StaticLogger::debug("Creating a class instance proxy from an existing jobject");
#endif //ENABLE_LOGGING

    Napi::Object jobject_wrapper = node_jobject_wrapper::createInstance();
    Napi::ObjectWrap<node_jobject_wrapper>::Unwrap(jobject_wrapper)->setData(obj);
#ifdef ENABLE_LOGGING
    StaticLogger::debug("Done setting the data");
#endif //ENABLE_LOGGING

    Napi::Function constructor = getConstructor(env, class_proxy);
    return constructor.New({jobject_wrapper});
}

java_instance_proxy::java_instance_proxy(const Napi::CallbackInfo &info) : ObjectWrap(info) {
    Napi::Object class_proxy_instance = info.NewTarget().ToObject().Get("class.proxy.instance").ToObject();
    java_class_proxy *class_ptr = Napi::ObjectWrap<java_class_proxy>::Unwrap(class_proxy_instance);

#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Creating a new '" << class_ptr->classname << "' instance";
#endif //ENABLE_LOGGING
    classname = class_ptr->classname;

    {
        std::unique_lock lock(class_ptr->mtx);
        clazz = class_ptr->clazz;
    }

#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Setting getters and setters for " << clazz->fields.size() << " fields";
#endif //ENABLE_LOGGING
    for (const auto &f: clazz->fields) {
        const auto getter = [f, this](const Napi::CallbackInfo &info) -> Napi::Value {
            TRY
                jni::jobject_wrapper<jobject> tmp;
                return conversion_helper::jvalue_to_napi_value(f.second.get(object, tmp), f.second.signature,
                                                               info.Env());
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
                    std::vector<jni::jobject_wrapper<jobject>> tmp;
                    f.second.set(object,
                                 conversion_helper::napi_value_to_jvalue(info.Env(), info[0], f.second.signature, tmp,
                                                                         true));
                CATCH_EXCEPTIONS
            };

            Value().DefineProperty(Napi::PropertyDescriptor::Accessor(f.first, getter, setter, napi_enumerable));
        }
    }

#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Setting functions for " << clazz->functions.size() << " java functions";
#endif //ENABLE_LOGGING
    for (const auto &f: clazz->functions) {
        const auto function = [f, this](const Napi::CallbackInfo &info) -> Napi::Value {
#ifdef ENABLE_LOGGING
            StaticLogger::debugStream << "Calling instance method '" << f.first << "' with " << info.Length()
                                      << " argument(s)";
#endif //ENABLE_LOGGING
            TRY
                return conversion_helper::call_matching_function(info, object, f.second);
            CATCH_EXCEPTIONS
        };

        const auto asyncFunction = [f, this](const Napi::CallbackInfo &info) -> Napi::Value {
#ifdef ENABLE_LOGGING
            StaticLogger::debugStream << "Calling instance method '" << f.first << "' with " << info.Length()
                                      << " argument(s) (async)";
#endif //ENABLE_LOGGING
            std::vector<jni::jobject_wrapper<jobject>> args;
            std::string error;
            std::vector<jvalue> values;
            auto *func = conversion_helper::find_matching_function(info, f.second, args, error, values);

            return napi_tools::promises::promise<jvalue_converter>(info.Env(), [args, error, values, func, this] {
                if (func == nullptr) {
                    throw std::runtime_error(error);
                }

                jvalue val = conversion_helper::call_function(*func, object, values);
                return jvalue_converter(val, func->returnType);
            });
        };

        Value().DefineProperty(Napi::PropertyDescriptor::Function(f.first, asyncFunction, napi_enumerable));
        Value().DefineProperty(Napi::PropertyDescriptor::Function(f.first + "Sync", function, napi_enumerable));
    }

    if (info.Length() == 1 && info[0].IsObject() && node_jobject_wrapper::instanceOf(info[0].ToObject())) {
#ifdef ENABLE_LOGGING
        StaticLogger::debug("The class constructor was called with a node_jobject_wrapper as first argument, "
                            "creating the class using that information");
#endif //ENABLE_LOGGING
        object = Napi::ObjectWrap<node_jobject_wrapper>::Unwrap(info[0].ToObject())->getObject();
    } else {
#ifdef ENABLE_LOGGING
        StaticLogger::debugStream << "Trying to find a matching constructor for the " << info.Length()
                                  << " provided arguments";
#endif //ENABLE_LOGGING
        std::vector<jni::jobject_wrapper<jobject>> outArgs;
        std::string error;
        const jni::java_constructor *constructor = conversion_helper::find_matching_constructor(info,
                                                                                                clazz->constructors,
                                                                                                outArgs, error);

        if (constructor == nullptr) {
            throw Napi::TypeError::New(info.Env(), error);
        } else {
            TRY
                object = constructor->newInstance(outArgs);
            CATCH_EXCEPTIONS
        }
    }
}

java_instance_proxy::~java_instance_proxy() {
#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Deleting class instance: " << classname;
#endif //ENABLE_LOGGING
    java_class_proxy::cleanup_class(clazz, classname);
}

void java_instance_proxy::Finalize(Napi::Env env) {
#ifdef ENABLE_LOGGING
    StaticLogger::debugStream << "Deleting class instance (finalize): " << classname;
#endif //ENABLE_LOGGING
}
