#include "util/util.hpp"

#include "jvm_lib/java_type.hpp"

java_type java_type::to_java_type(std::string signature, bool convert) {
    if (convert) {
        signature = util::make_java_name_readable(signature);
    }

    if (signature == "void") {
        return {Void, nullptr, signature};
    } else if (signature == "int") {
        return {Integer, nullptr, signature};
    } else if (signature == "boolean") {
        return {Boolean, nullptr, signature};
    } else if (signature == "byte") {
        return {Byte, nullptr, signature};
    } else if (signature == "char") {
        return {Character, nullptr, signature};
    } else if (signature == "short") {
        return {Short, nullptr, signature};
    } else if (signature == "long") {
        return {Long, nullptr, signature};
    } else if (signature == "float") {
        return {Float, nullptr, signature};
    } else if (signature == "double") {
        return {Double, nullptr, signature};
    } else if (signature == "java.lang.Integer") {
        return {lang_Integer, nullptr, signature};
    } else if (signature == "java.lang.Boolean") {
        return {lang_Boolean, nullptr, signature};
    } else if (signature == "java.lang.Byte") {
        return {lang_Byte, nullptr, signature};
    } else if (signature == "java.lang.Character") {
        return {lang_Character, nullptr, signature};
    } else if (signature == "java.lang.Short") {
        return {lang_Short, nullptr, signature};
    } else if (signature == "java.lang.Long") {
        return {lang_Long, nullptr, signature};
    } else if (signature == "java.lang.Float") {
        return {lang_Float, nullptr, signature};
    } else if (signature == "java.lang.Double") {
        return {lang_Double, nullptr, signature};
    } else if (signature == "java.lang.String") {
        return {String, nullptr, signature};
    } else if (util::hasEnding(signature, "[]")) {
        return {Array,
                std::make_shared<java_type>(to_java_type(signature.substr(0, signature.size() - 2), false)),
                signature};
    } else if (signature == "java.lang.Object") {
        return {lang_Object, nullptr, signature};
    } else {
        return {Object, nullptr, signature};
    }
}

java_type::java_type() : type(), inner(nullptr), signature() {}

java_type::java_type(j_type type, std::shared_ptr<java_type> inner, std::string signature)
        : type(type), inner(std::move(inner)), signature(std::move(signature)) {}

bool java_type::operator==(std::uint8_t t) const {
    return type == t;
}

bool java_type::operator!=(std::uint8_t t) const {
    return type != t;
}

bool java_type::isVoid() const {
    return type == j_type::Void;
}

bool java_type::isInt() const {
    return type == j_type::Integer;
}

bool java_type::isBool() const {
    return type == j_type::Boolean;
}

bool java_type::isByte() const {
    return type == j_type::Byte;
}

bool java_type::isChar() const {
    return type == j_type::Character;
}

bool java_type::isShort() const {
    return type == j_type::Short;
}

bool java_type::isLong() const {
    return type == j_type::Long;
}

bool java_type::isFloat() const {
    return type == j_type::Float;
}

bool java_type::isDouble() const {
    return type == j_type::Double;
}

bool java_type::isArray() const {
    return type == j_type::Array;
}

bool java_type::isPrimitive() const {
    return isInt() || isBool() || isByte() || isChar() || isShort() || isLong() || isFloat() || isDouble();
}
