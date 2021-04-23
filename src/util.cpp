#include <stdexcept>

#include "util.hpp"
#include "definitions.hpp"

#ifdef JAVA_WINDOWS
#   define CP_DELIMITER ';'
#else
#   define CP_DELIMITER ':'
#endif //JAVA_WINDOWS

std::string util::jni_error_to_string(jint code) {
    switch (code) {
        case JNI_EDETACHED:
            return "Thread detached from the vm";
        case JNI_EVERSION:
            return "JNI version error";
        case JNI_ENOMEM:
            return "Not enough memory";
        case JNI_EEXIST:
            return "VM already created";
        case JNI_EINVAL:
            return "Invalid arguments";
        default:
            return "Unknown error";
    }
}

std::string util::java_type_to_jni_type(const std::string &to_convert) {
    if (to_convert == "boolean") {
        return "Z";
    } else if (to_convert == "byte") {
        return "B";
    } else if (to_convert == "char") {
        return "C";
    } else if (to_convert == "short") {
        return "S";
    } else if (to_convert == "int") {
        return "I";
    } else if (to_convert == "long") {
        return "J";
    } else if (to_convert == "float") {
        return "F";
    } else if (to_convert == "double") {
        return "D";
    } else if (to_convert == "void") {
        return "V";
    } else {
        if (!to_convert.empty() && to_convert[0] != '[' && to_convert[0] != 'L') {
            return 'L' + string_replace(to_convert, '.', '/') + ';';
        } else {
            return string_replace(to_convert, '.', '/');
        }
    }
}

std::string util::string_replace(std::string val, char to_replace, char replacement) {
    for (char &c : val) {
        if (c == to_replace) {
            c = replacement;
        }
    }

    return val;
}

jint util::string_to_java_version(const std::string &ver) {
    if (ver == "1.1") {
        return JNI_VERSION_1_1;
    } else if (ver == "1.2") {
        return JNI_VERSION_1_2;
    } else if (ver == "1.4") {
        return JNI_VERSION_1_4;
    } else if (ver == "1.6") {
        return JNI_VERSION_1_6;
    } else if (ver == "1.8") {
        return JNI_VERSION_1_8;
    } else if (ver == "9") {
        return JNI_VERSION_9;
    } else if (ver == "10") {
        return JNI_VERSION_10;
    } else {
        throw std::runtime_error("Invalid version string supplied: " + ver);
    }
}

std::string util::classpath_elements_to_classpath(const std::vector<std::string> &elements) {
    std::string res;
    res += "-Djava.class.path=";

    for (size_t i = 0; i < elements.size(); i++) {
        if (i > 0) {
            res += CP_DELIMITER;
        }

        res += elements[i];
    }

    return res;
}