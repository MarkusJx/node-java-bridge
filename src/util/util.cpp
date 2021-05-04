#include <stdexcept>

#include "util/util.hpp"
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
        if (hasEnding(to_convert, "[]")) {
            return "[" + java_type_to_jni_type(to_convert.substr(to_convert.size() - 2));
        } else if (!to_convert.empty() && to_convert[0] != '[' && to_convert[0] != 'L') {
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

std::string util::make_java_name_readable(const std::string &to_convert) {
    if (to_convert == "Z") {
        return "java.lang.Boolean";
    } else if (to_convert == "B") {
        return "java.lang.Byte";
    } else if (to_convert == "C") {
        return "java.lang.Character";
    } else if (to_convert == "S") {
        return "java.lang.Short";
    } else if (to_convert == "I") {
        return "java.lang.Integer";
    } else if (to_convert == "J") {
        return "java.lang.Long";
    } else if (to_convert == "F") {
        return "java.lang.Float";
    } else if (to_convert == "D") {
        return "java.lang.Double";
    } else if (to_convert == "V") {
        return "void";
    } else if (!to_convert.empty() && to_convert[0] == '[') {
        return make_java_name_readable(to_convert.substr(1)) + "[]";
    } else if (!to_convert.empty() && to_convert[0] == 'L') {
        return string_replace(to_convert.substr(1, to_convert.size() - 2), '/', '.');
    } else {
        return to_convert;
    }
}

std::string util::get_java_version_from_jint(jint version) {
    // The major version will be in the higher
    // 16 bits and the minor version will
    // be in the lower 16 bits

    // Shift 16 bits to the right
    const jint hi = version >> 16;

    // Shift 16 bits to the left to discard
    // the major version bits, then shift
    // 16 bits to the right to move everything back
    const jint lo = (version << 16) >> 16;

    return std::to_string(hi) + "." + std::to_string(lo);
}

bool util::hasEnding(std::string const &fullString, std::string const &ending) {
    if (fullString.length() >= ending.length()) {
        return (0 == fullString.compare (fullString.length() - ending.length(), ending.length(), ending));
    } else {
        return false;
    }
}