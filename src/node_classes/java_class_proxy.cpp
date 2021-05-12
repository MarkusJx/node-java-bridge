#include "node_classes/conversion_helper.hpp"
#include "node_classes/java_class_proxy.hpp"
#include "node_classes/java_instance_proxy.hpp"

#include <napi_tools.hpp>
#ifdef ENABLE_LOGGING
#   include <logger.hpp>
#endif //ENABLE_LOGGING
#include <node_classes/jvm_container.hpp>

using namespace node_classes;
#ifdef ENABLE_LOGGING
using namespace markusjx::logging;
#endif //ENABLE_LOGGING

/**
 * A helper class which will create
 * the class proxy in a promise
 */
class proxy_creator {
public:
    proxy_creator() = default;

    explicit proxy_creator(std:: string classname) : classname(std::move(classname)) {}

    [[maybe_unused]] static Napi::Value toNapiValue(const Napi::Env &env, const proxy_creator &c) {
        return java_class_proxy::createInstance(Napi::String::New(env, c.classname));
    }

private:
    std::string classname;
};

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

Napi::Value java_class_proxy::createInstanceAsync(const std::string &classname, const Napi::Env &env) {
    std::unique_lock<std::mutex> lock(cache_mtx);
    if (cached_classes.find(classname) != cached_classes.end()) {
        lock.unlock();
        // The class is already cached,
        // just return the class instance
        return constructor->New({Napi::String::New(env, classname)});
    } else {
        lock.unlock();
        return napi_tools::promises::promise<proxy_creator>(env, [classname] {
            jni::jni_wrapper jvm = node_classes::jvm_container::attachJvm();
            auto j_class = std::make_shared<jni::java_class>(jvm.getClass(classname));

            // Just add the resolved class to the class cache,
            // the constructor will find and use it
            {
                std::unique_lock<std::mutex> lock(cache_mtx);
                cached_classes.insert(std::pair<std::string, std::shared_ptr<jni::java_class>>(classname, j_class));
            }

            return proxy_creator(classname);
        });
    }
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
#ifdef ENABLE_LOGGING
        StaticLogger::debugStream << "Creating a new class proxy instance for class " << classname;
#endif //ENABLE_LOGGING

        // Get our very own java environment pointer
        jni::jni_wrapper jvm = node_classes::jvm_container::attachJvm();
        std::unique_lock<std::mutex> cache_lock(cache_mtx);
        if (cached_classes.find(classname) != cached_classes.end()) {
            clazz = cached_classes.at(classname);
        } else {
            clazz = std::make_shared<jni::java_class>(jvm.getClass(classname));
            cached_classes.insert(std::pair<std::string, std::shared_ptr<jni::java_class>>(classname, clazz));
        }

        Value().DefineProperty(Napi::PropertyDescriptor::Value("class.name", Napi::String::New(info.Env(), classname),
                                                               napi_enumerable));
    CATCH_EXCEPTIONS
}

java_class_proxy::~java_class_proxy() {
    cleanup_class(clazz, classname);
}

void java_class_proxy::cleanup_class(std::shared_ptr<jni::java_class> &ptr, const std::string &classname) {
    ptr.reset();
    std::unique_lock<std::mutex> lock(cache_mtx);
    auto iter = cached_classes.find(classname);
    if (iter != cached_classes.end() && iter->second.use_count() == 1) {
        // If the use count of the this's cached class is one
        // this was the last instance referencing the class,
        // therefore we can un-cache the class, deleting it.
        cached_classes.erase(iter);
    }
}

Napi::FunctionReference *java_class_proxy::constructor = nullptr;

std::mutex java_class_proxy::cache_mtx;
// This may throw an exception that can't be caught, but it's worth the risk, I guess.
std::map<std::string, std::shared_ptr<jni::java_class>> java_class_proxy::cached_classes;