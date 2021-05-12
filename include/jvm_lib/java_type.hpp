#ifndef NODE_JAVA_BRIDGE_JAVA_TYPE_HPP
#define NODE_JAVA_BRIDGE_JAVA_TYPE_HPP

#include <memory>
#include <string>

#include "definitions.hpp"

enum j_type: std::uint8_t {
    Void = 0,
    Object = 1,
    Array = 2,
    Integer = 3,
    Boolean = 4,
    Byte = 5,
    Character = 6,
    Short = 7,
    Long = 8,
    Float = 9,
    Double = 10,
    lang_Integer = 11,
    lang_Boolean = 12,
    lang_Byte = 13,
    lang_Character = 14,
    lang_Short = 15,
    lang_Long = 16,
    lang_Float = 17,
    lang_Double = 18,
    lang_Object = 19,
    String = 20
};

class java_type {
public:
    static java_type to_java_type(std::string signature, bool convert = true);

    java_type(j_type type, std::shared_ptr<java_type> inner, std::string signature);

    java_type();

    bool operator==(std::uint8_t t) const;

    bool operator!=(std::uint8_t t) const;

    JAVA_NODISCARD bool isVoid() const;

    JAVA_NODISCARD bool isInt() const;

    JAVA_NODISCARD bool isBool() const;

    JAVA_NODISCARD bool isByte() const;

    JAVA_NODISCARD bool isChar() const;

    JAVA_NODISCARD bool isShort() const;

    JAVA_NODISCARD bool isLong() const;

    JAVA_NODISCARD bool isFloat() const;

    JAVA_NODISCARD bool isDouble() const;

    JAVA_NODISCARD bool isArray() const;

    JAVA_NODISCARD bool isPrimitive() const;

    j_type type;
    std::shared_ptr<java_type> inner;
    std::string signature;
};

#endif //NODE_JAVA_BRIDGE_JAVA_TYPE_HPP
