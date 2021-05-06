#include <fstream>
#include <iostream>
#include <napi_tools.hpp>
#include <node_classes/java_instance_proxy.hpp>

#include "node_classes/jvm_container.hpp"
#include "node_classes/conversion_helper.hpp"
#include "definitions.hpp"

#include "jvm_lib/io_github_markusjx_bridge_JavaFunctionCaller.h"
#include "node_classes/java_function_caller.hpp"

using namespace node_classes;

class node_classes::java_function_caller::value_converter {
public:
    value_converter() = default;

    JAVA_UNUSED static value_converter fromNapiValue(const Napi::Value &val) {
        value_converter c;
        c.object = convert_value(val);

        return c;
    }

    jni::jobject_wrapper<jobject> object;

private:
    static jni::jobject_wrapper<jobject> convert_value(const Napi::Value &value) {
        jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();

        if (value.IsNull() || value.IsUndefined()) {
            return jni::jobject_wrapper();
        } else if (value.IsNumber()) {
            return j_env.create_jint(value.ToNumber().Int32Value());
        } else if (value.IsBigInt()) {
            bool lossless;
            return j_env.create_jlong(value.As<Napi::BigInt>().Int64Value(&lossless));
        } else if (value.IsBoolean()) {
            return j_env.create_jboolean(value.ToBoolean().Value());
        } else if (value.IsString()) {
            return j_env.string_to_jstring(value.ToString().Utf8Value()).as<jobject>();
        } else if (value.IsArray()) {
            auto array = value.As<Napi::Array>();
            jclass clazz = j_env.getJClass("java.lang.Object");
            jint array_size = static_cast<jint>(array.Length());

            jni::jobject_wrapper<jobjectArray> j_array(j_env->NewObjectArray(array_size, clazz, nullptr), j_env);
            j_env.checkForError();

            for (jint i = 0; i < array_size; i++) {
                j_env->SetObjectArrayElement(j_array, i, convert_value(array.Get(i)));
                j_env.checkForError();
            }

            return j_array.as<jobject>();
        } else {
            Napi::Object obj = value.ToObject();
            if (node_classes::java_function_caller::instanceOf(obj)) {
                return Napi::ObjectWrap<node_classes::java_function_caller>::Unwrap(obj)->proxy;
            } else {
                auto *ptr = Napi::ObjectWrap<node_classes::java_instance_proxy>::Unwrap(obj);
                return ptr->object;
            }
        }
    }
};

JAVA_UNUSED jobject
Java_io_github_markusjx_bridge_JavaFunctionCaller_callNodeFunction(JNIEnv *env, jobject, jlong ptr, jobject method,
                                                                   jobjectArray args) {
    try {
        const auto caller = (java_function_caller *) ptr;

        std::vector<napi_value> values;
        if (args != nullptr) {
            const jsize numArgs = env->GetArrayLength(args);
            values.resize(numArgs);
            for (jsize i = 0; i < numArgs; i++) {
                jni::local_jobject obj(env->GetObjectArrayElement(args, i));
                values[i] = conversion_helper::jobject_to_value(caller->Env(), obj, "java.lang.Object");
            }
        }

        jclass Method = env->GetObjectClass(method);
        jmethodID getName = env->GetMethodID(Method, "getName", "()Ljava/lang/String;");

        auto j_name = (jstring) env->CallObjectMethod(method, getName);
        const char *chars = env->GetStringUTFChars(j_name, nullptr);

        std::string name(chars);
        env->ReleaseStringUTFChars(j_name, chars);

        java_function_caller::value_converter res = caller->functions.at(name).callSync(args);
        if (res.object.isNull()) {
            return nullptr;
        } else {
            return env->NewLocalRef(res.object);
        }
    } catch (const std::exception &e) {
        env->ThrowNew(env->FindClass("java/lang/Exception"), e.what());
        return nullptr;
    }
}

const char *JAVA_CLASS_PATH = "/io/github/markusjx/bridge/JavaFunctionCaller.class";

void java_function_caller::setLibraryPath(const std::string &path, const std::string &workingDir) {
    nativeLibPath = path;

    std::ifstream file(workingDir + JAVA_CLASS_PATH, std::ios::in | std::ios::binary | std::ios::ate);
    std::streamsize size = file.tellg();
    file.seekg(0, std::ios::beg);

    std::vector<char> buffer(size);
    if (file.read(buffer.data(), size)) {
        classData = buffer;
    } else {
        throw std::runtime_error("Could not load the JavaFunctionCaller.class file");
    }
    file.close();
}

bool java_function_caller::instanceOf(const Napi::Object &object) {
    if (constructor == nullptr) {
        return false;
    } else {
        return object.InstanceOf(constructor->Value());
    }
}

void java_function_caller::init(Napi::Env env, Napi::Object &exports) {
    Napi::Function func = DefineClass(env, "java_function_caller", {});

    constructor = new Napi::FunctionReference();
    *constructor = Napi::Persistent(func);

    exports.Set("java_function_caller", func);
    env.SetInstanceData<Napi::FunctionReference>(constructor);
}

java_function_caller::java_function_caller(const Napi::CallbackInfo &info) : ObjectWrap(info), functions() {
    CHECK_ARGS(napi_tools::string, napi_tools::object);
    jni::jni_wrapper jvm = node_classes::jvm_container::attachJvm();

    TRY
        clazz = jvm->DefineClass("io/github/markusjx/bridge/JavaFunctionCaller", nullptr,
                                 reinterpret_cast<const jbyte *>(classData.data()),
                                 static_cast<jsize>(classData.size()));
        jvm.checkForError();

        Napi::Object obj = info[1].ToObject();
        Napi::Array names = obj.GetPropertyNames();

        jobjectArray arr = jvm->NewObjectArray((jsize) names.Length(), jvm->FindClass("java/lang/String"), nullptr);
        jvm.checkForError();

        const auto converter = [](const Napi::Env &env, jobjectArray args) -> std::vector<napi_value> {
            return {};
        };

        for (uint32_t i = 0; i < names.Length(); i++) {
            Napi::String name = names.Get(i).ToString();
            if (obj.Get(name).IsFunction()) {
                callback c(info.Env(), obj.Get(name).As<Napi::Function>(), converter);
                functions.insert(std::pair<std::string, callback>(name.Utf8Value(), c));
                jvm->SetObjectArrayElement(arr, (jsize) i, jvm.string_to_jstring(name.Utf8Value()));
            } else {
                throw Napi::TypeError::New(info.Env(), "All values in the object must be functions");
            }
        }

        std::string classname = info[0].ToString().Utf8Value();

        jmethodID ctor = jvm->GetMethodID(clazz, "<init>", "(Ljava/lang/String;[Ljava/lang/String;J)V");
        jvm.checkForError();
        object = jni::jobject_wrapper(
                jvm->NewObject(clazz, ctor, jvm.string_to_jstring(nativeLibPath).obj, arr,
                               (jlong) this), jvm);
        jvm->DeleteLocalRef(arr);

        jclass Proxy = jvm->FindClass("java/lang/reflect/Proxy");
        jmethodID newProxyInstance = jvm->GetStaticMethodID(Proxy, "newProxyInstance",
                                                            "(Ljava/lang/ClassLoader;[Ljava/lang/Class;Ljava/lang/reflect/InvocationHandler;)Ljava/lang/Object;");

        jobjectArray classes = jvm->NewObjectArray(1, jvm.getJavaLangClass(), jvm.getClassByName(classname).obj);
        proxy = jni::jobject_wrapper(
                jvm->CallStaticObjectMethod(Proxy, newProxyInstance, jvm.getClassloader().obj, classes, object.obj),
                jvm);
        jvm->DeleteLocalRef(classes);
    CATCH_EXCEPTIONS
}

java_function_caller::~java_function_caller() {
    try {
        jni::jni_wrapper jvm = node_classes::jvm_container::attachJvm();
        jmethodID destruct = jvm->GetMethodID(clazz, "destruct", "()V");
        jvm.checkForError();
        jvm->CallVoidMethod(object, destruct);
        jvm.checkForError();
    } catch (const std::exception &e) {
        std::cerr << e.what() << std::endl;
    }
}

Napi::FunctionReference *java_function_caller::constructor = nullptr;
std::string java_function_caller::nativeLibPath;
std::vector<char> java_function_caller::classData;