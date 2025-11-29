// Tests for Asm::Dest/Comp/Jump API
#include <gtest/gtest.h>
#include <string>

#include "../code.h"

static std::string to_string(const std::array<char, 4>& a)
{
    return std::string(a.data());
}

static std::string to_string8(const std::array<char, 8>& a)
{
    return std::string(a.data());
}

TEST(CodeTest, DestKnown)
{
    EXPECT_EQ(to_string(Asm::Dest("D")), "010");
    EXPECT_EQ(to_string(Asm::Dest("M")), "001");
    EXPECT_EQ(to_string(Asm::Dest("ADM")), "111");
    EXPECT_EQ(to_string(Asm::Dest("null")), "000");
}

TEST(CodeTest, JumpKnown)
{
    EXPECT_EQ(to_string(Asm::Jump("JMP")), "111");
    EXPECT_EQ(to_string(Asm::Jump("JGT")), "001");
    EXPECT_EQ(to_string(Asm::Jump("null")), "000");
}

TEST(CodeTest, CompKnown)
{
    EXPECT_EQ(to_string8(Asm::Comp("A")), "0110000");
    EXPECT_EQ(to_string8(Asm::Comp("D")), "0001100");
    EXPECT_EQ(to_string8(Asm::Comp("1")), "1111111");
    EXPECT_EQ(to_string8(Asm::Comp("0")), "101010");
    EXPECT_EQ(to_string8(Asm::Comp("M")), "1110000");
}

TEST(CodeTest, UnknownMnemonic)
{
    EXPECT_TRUE(to_string(Asm::Dest("XYZ")).empty());
    EXPECT_TRUE(to_string8(Asm::Comp("XYZ")).empty());
    EXPECT_TRUE(to_string(Asm::Jump("XYZ")).empty());
}
