#include <fstream>
#include <iostream>

#include "node_classes/java_instance_proxy.hpp"
#include "node_classes/jvm_container.hpp"
#include "node_classes/conversion_helper.hpp"
#include "definitions.hpp"

#include "jvm_lib/io_github_markusjx_bridge_JavaFunctionCaller.h"
#include "node_classes/java_function_caller.hpp"

#ifdef ENABLE_LOGGING
#   include <logger.hpp>
#endif //ENABLE_LOGGING

using namespace node_classes;

std::vector<java_function_caller *> active_proxies;

void add_proxy(java_function_caller *proxy) {
    active_proxies.push_back(proxy);
}

bool proxy_exists(java_function_caller *proxy) {
    return std::find(active_proxies.begin(), active_proxies.end(), proxy) != active_proxies.end();
}

void remove_proxy(java_function_caller *proxy) {
    auto to_delete = std::find(active_proxies.begin(), active_proxies.end(), proxy);
    if (to_delete != active_proxies.end()) {
        active_proxies.erase(to_delete);
    }
}

class node_classes::java_function_caller::value_converter {
public:
    value_converter() = default;

    JAVA_UNUSED static value_converter fromNapiValue(const Napi::Env &env, const Napi::Value &val) {
        value_converter c;
        try {
            if (val.IsNull() || val.IsUndefined()) {
                c.object = jni::jobject_wrapper();
            } else {
                c.object = conversion_helper::value_to_jobject(env, val, java_type(j_type::lang_Object, nullptr,
                                                                                   "java.lang.Object"), true);
            }
        } catch (const std::exception &e) {
            std::cerr << __FILE__ << ':' << __LINE__ << " " << e.what() << std::endl;
            c.object = jni::jobject_wrapper();
        }

        return c;
    }

    jni::jobject_wrapper<jobject> object;
};

jthrowable jsErrorToException(JNIEnv *env, napi_tools::exception &e) {
    const auto check_exception = [&env](jstring msg, jobjectArray stackTrace, jstring str) {
        if (env->ExceptionCheck()) {
            env->ExceptionClear();
            if (msg) env->DeleteLocalRef(msg);
            if (stackTrace) env->DeleteLocalRef(stackTrace);
            if (str) env->DeleteLocalRef(str);
            throw std::runtime_error("Could not convert the javascript exception");
        }
    };

    jclass utils = env->FindClass("io/github/markusjx/bridge/Util");
    check_exception(nullptr, nullptr, nullptr);

    jmethodID exceptionFromJsError = env->GetStaticMethodID(utils, "exceptionFromJsError",
                                                            "(Ljava/lang/String;[Ljava/lang/String;)Ljava/lang/Exception;");
    check_exception(nullptr, nullptr, nullptr);

    e.add_to_stack(__FUNCTION__, __FILE__, __LINE__);
    const size_t sz = e.stack().size();
    const auto msg = env->NewStringUTF(e.what());
    const auto stackTrace = env->NewObjectArray(static_cast<jsize>(sz), env->FindClass("java/lang/String"), nullptr);
    check_exception(msg, stackTrace, nullptr);

    for (size_t i = 0; i < sz; i++) {
        auto str = env->NewStringUTF(e.stack().at(i).c_str());
        check_exception(msg, stackTrace, str);

        env->SetObjectArrayElement(stackTrace, static_cast<jsize>(i), str);
        check_exception(msg, stackTrace, str);

        env->DeleteLocalRef(str);
        check_exception(msg, stackTrace, nullptr);
    }

    jobject exception = env->CallStaticObjectMethod(utils, exceptionFromJsError, msg, stackTrace);
    check_exception(msg, stackTrace, nullptr);

    env->DeleteLocalRef(msg);
    env->DeleteLocalRef(stackTrace);

    return (jthrowable) exception;
}

JAVA_UNUSED jobject
Java_io_github_markusjx_bridge_JavaFunctionCaller_callNodeFunction(JNIEnv *env, jobject, jlong ptr, jobject method,
                                                                   jobjectArray args) {
    try {
        const auto caller = (java_function_caller *) ptr;

        if (!proxy_exists(caller)) {
            env->ThrowNew(env->FindClass("java/lang/Exception"), "No javascript proxy with the given address exists");
            return nullptr;
        } else if (caller->is_destroyed()) {
            env->ThrowNew(env->FindClass("java/lang/Exception"), "The javascript proxy has been destroyed");
            return nullptr;
        }

        // Get the name of the method to invoke
        jclass Method = env->GetObjectClass(method);
        jmethodID getName = env->GetMethodID(Method, "getName", "()Ljava/lang/String;");

        auto j_name = (jstring) env->CallObjectMethod(method, getName);
        const char *chars = env->GetStringUTFChars(j_name, nullptr);

        std::string name(chars);
        env->ReleaseStringUTFChars(j_name, chars);

        // Call the js function and await the result
        java_function_caller::value_converter res = caller->functions.at(name).callSync(args, env);
        if (!res.object.isNull()) {
            return env->NewLocalRef(res.object);
        }
    } catch (napi_tools::exception &e) {
        try {
            e.add_to_stack(__FUNCTION__, __FILE__, __LINE__);
            env->Throw(jsErrorToException(env, e));
        } catch (const std::exception &e) {
            env->ThrowNew(env->FindClass("java/lang/Exception"), e.what());
        }
    } catch (const std::exception &e) {
        env->ThrowNew(env->FindClass("java/lang/Exception"), e.what());
    }

    return nullptr;
}

/**
 * Convert a java argument array to a vector of napi_values
 *
 * @param env the n-api environment to use
 * @param args the arguments to convert
 * @param jniEnv the jni environment to use
 * @return the converted values
 */
std::vector<napi_value> convert_object(const Napi::Env &env, jobjectArray args, JNIEnv *jniEnv) {
    try {
        std::vector<napi_value> values;
        if (args != nullptr) {
            const jsize numArgs = jniEnv->GetArrayLength(args);
            values.resize(numArgs);
            for (jsize i = 0; i < numArgs; i++) {
                jni::local_jobject obj(jniEnv->GetObjectArrayElement(args, i));
                values[i] = conversion_helper::jobject_to_value(env, obj, java_type(j_type::lang_Object, nullptr,
                                                                                    "java.lang.Object"));
            }
        }

        return values;
    } catch (const std::exception &e) {
        jniEnv->ThrowNew(jniEnv->FindClass("java/lang/Exception"), e.what());
        return {};
    }
}

bool java_function_caller::instanceOf(const Napi::Object &object) {
    if (constructor == nullptr) {
        return false;
    } else {
        return object.InstanceOf(constructor->Value());
    }
}

void java_function_caller::init(Napi::Env &env, Napi::Object &exports) {
    Napi::Function func = DefineClass(env, "java_function_caller", {
            InstanceMethod("destroy", &java_function_caller::destroy_instance, napi_enumerable),
    });

    constructor = new Napi::FunctionReference();
    *constructor = Napi::Persistent(func);

    exports.Set("java_function_caller", func);
    env.SetInstanceData<Napi::FunctionReference>(constructor);
}

java_function_caller::java_function_caller(const Napi::CallbackInfo &info) : ObjectWrap(info), functions(),
                                                                             destroyed(false) {
    CHECK_ARGS(napi_tools::string, napi_tools::object);

    TRY
        jni::jni_wrapper jvm = node_classes::jvm_container::attachJvm();
        clazz = jvm.getJClass("io.github.markusjx.bridge.JavaFunctionCaller");
        jvm.checkForError();

        Napi::Object obj = info[1].ToObject();
        Napi::Array names = obj.GetPropertyNames();

        jobjectArray arr = jvm->NewObjectArray((jsize) names.Length(), jvm->FindClass("java/lang/String"), nullptr);
        jvm.checkForError();

        for (uint32_t i = 0; i < names.Length(); i++) {
            Napi::String name = names.Get(i).ToString();
            if (obj.Get(name).IsFunction()) {
                callback c(info.Env(), obj.Get(name).As<Napi::Function>(), convert_object);
                functions.insert(std::pair<std::string, callback>(name.Utf8Value(), c));
                jvm->SetObjectArrayElement(arr, (jsize) i, jvm.string_to_jstring(name.Utf8Value()));
            } else {
                throw Napi::TypeError::New(info.Env(), "All values in the object must be functions");
            }
        }

        classname = info[0].ToString().Utf8Value();

        jmethodID ctor = jvm->GetMethodID(clazz, "<init>", "([Ljava/lang/String;J)V");
        jvm.checkForError();
        object = jni::jobject_wrapper(jvm->NewObject(clazz, ctor, arr, (jlong) this), jvm);
        jvm->DeleteLocalRef(arr);

        jclass Proxy = jvm->FindClass("java/lang/reflect/Proxy");
        jmethodID newProxyInstance = jvm->GetStaticMethodID(Proxy, "newProxyInstance",
                                                            "(Ljava/lang/ClassLoader;[Ljava/lang/Class;Ljava/lang/reflect/InvocationHandler;)Ljava/lang/Object;");

        jobjectArray classes = jvm->NewObjectArray(1, jvm.getJavaLangClass(), jvm.getClassByName(classname).obj);
        proxy = jni::jobject_wrapper(
                jvm->CallStaticObjectMethod(Proxy, newProxyInstance, jvm.getClassloader().obj, classes, object.obj),
                jvm);
        jvm->DeleteLocalRef(classes);

        add_proxy(this);
    CATCH_EXCEPTIONS
}

const std::string &java_function_caller::getClassName() const {
    return classname;
}

Napi::Value java_function_caller::destroy_instance(const Napi::CallbackInfo &info) {
    return napi_tools::promises::promise<void>(info.Env(), [this] {
        if (is_destroyed()) {
            throw std::runtime_error("The proxy has already been destroyed");
        }

        destroy();
        object.reset();
    });
}

bool java_function_caller::is_destroyed() const {
    return destroyed;
}

void java_function_caller::destroy() {
    if (!is_destroyed()) {
        destroyed = true;

#ifdef ENABLE_LOGGING
        markusjx::logging::StaticLogger::debugStream << "Destroying function caller for class: " << classname;
#endif //ENABLE_LOGGING

        jni::jni_wrapper jvm = node_classes::jvm_container::attachJvm();
        jmethodID destruct = jvm->GetMethodID(clazz, "destruct", "()V");
        jvm.checkForError();
        jvm->CallVoidMethod(object, destruct);
        jvm.checkForError();

#ifdef ENABLE_LOGGING
        markusjx::logging::StaticLogger::debugStream << "function caller for class '" << classname << "' destroyed";
#endif //ENABLE_LOGGING
    }
}

java_function_caller::~java_function_caller() {
    try {
        remove_proxy(this);
        destroy();
    } catch (const std::exception &e) {
        std::cerr << e.what() << std::endl;
    }
}

Napi::FunctionReference *java_function_caller::constructor = nullptr;