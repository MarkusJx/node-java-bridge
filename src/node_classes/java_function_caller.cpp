#include <fstream>
#include <iostream>

#include "node_classes/java_instance_proxy.hpp"
#include "node_classes/jvm_container.hpp"
#include "node_classes/conversion_helper.hpp"
#include "definitions.hpp"

#include "jvm_lib/io_github_markusjx_bridge_JavaFunctionCaller.h"
#include "node_classes/java_function_caller.hpp"

using namespace node_classes;

const char *JAVA_CLASS_PATH = "/io/github/markusjx/bridge/JavaFunctionCaller.class";

class node_classes::java_function_caller::value_converter {
public:
    value_converter() = default;

    JAVA_UNUSED static value_converter fromNapiValue(const Napi::Env &env, const Napi::Value &val) {
        value_converter c;
        try {
            if (val.IsNull() || val.IsUndefined()) {
                c.object = jni::jobject_wrapper();
            } else {
                c.object = conversion_helper::value_to_jobject(env, val, "java.lang.Object", true);
            }
        } catch (const std::exception &e) {
            std::cerr << __FILE__ << ':' << __LINE__ << " " << e.what() << std::endl;
            c.object = jni::jobject_wrapper();
        }

        return c;
    }

    jni::jobject_wrapper<jobject> object;
};

JAVA_UNUSED jobject
Java_io_github_markusjx_bridge_JavaFunctionCaller_callNodeFunction(JNIEnv *env, jobject, jlong ptr, jobject method,
                                                                   jobjectArray args) {
    try {
        const auto caller = (java_function_caller *) ptr;

        // Get the name of the method to invoke
        jclass Method = env->GetObjectClass(method);
        jmethodID getName = env->GetMethodID(Method, "getName", "()Ljava/lang/String;");

        auto j_name = (jstring) env->CallObjectMethod(method, getName);
        const char *chars = env->GetStringUTFChars(j_name, nullptr);

        std::string name(chars);
        env->ReleaseStringUTFChars(j_name, chars);

        // Call the js function and await the result
        java_function_caller::value_converter res = caller->functions.at(name).callSync(args, env);
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
                values[i] = conversion_helper::jobject_to_value(env, obj, "java.lang.Object");
            }
        }

        return values;
    } catch (const std::exception &e) {
        jniEnv->ThrowNew(jniEnv->FindClass("java/lang/Exception"), e.what());
        return std::vector<napi_value>();
    }
}

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

void java_function_caller::init(Napi::Env &env, Napi::Object &exports) {
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