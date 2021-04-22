//#include <napi.h>
#include <iostream>
#include <jni.h>

#include "shared_library.hpp"
#include "jni_wrapper.hpp"

#define JVM_DLL_PATH R"(C:\Program Files\AdoptOpenJDK\jdk-11.0.10.9-hotspot\bin\client\jvm.dll)"

int main() {
    try {
        JavaVMInitArgs vm_args;
        auto* options = new JavaVMOption[0];
        //options[0].optionString = "-Djava.class.path=/usr/lib/java";
        vm_args.version = JNI_VERSION_1_6;
        vm_args.nOptions = 0;
        vm_args.options = options;
        vm_args.ignoreUnrecognized = false;
        jni_wrapper jvm(JVM_DLL_PATH, vm_args);

        jobject_wrapper str = jvm.string_to_jstring("abc");
        std::cout << jvm.jstring_to_string(str) << ", classname: " << jvm.get_object_class_name(str) << std::endl;

        jvm->FindClass("java/lang/String");
        if (jvm->ExceptionCheck()) {
            std::cerr << jvm.getLastException().what() << std::endl;
        }

        auto constructors = jvm.getClassConstructors("java.lang.String");
        for (const auto &ctor : constructors) {
            std::cout << ctor.to_string() << ", num args: " << ctor.numArguments() << ", types:" << std::endl;
            for (const auto &param : ctor.getParameterTypes()) {
                std::cout << '\t' << param << std::endl;
            }
        }
    } catch (const std::exception &e) {
        std::cerr << "Exception thrown: " << e.what() << std::endl;
    }

    return 0;
}

/*Napi::Object InitAll(Napi::Env env, Napi::Object exports) {

    return exports;
}

NODE_API_MODULE(node_java_bridge, InitAll)*/