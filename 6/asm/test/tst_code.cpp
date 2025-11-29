// Tests for Asm::Dest/Comp/Jump API
#include <gtest/gtest.h>
#include <string>

#include "../code.h"

TEST(CodeTest, DestKnown)
{
    EXPECT_EQ(Asm::Dest("D"), "010");
    EXPECT_EQ(Asm::Dest("M"), "001");
    EXPECT_EQ(Asm::Dest("ADM"), "111");
    // unknown destination or explicit "null" should produce "000"
    EXPECT_EQ(Asm::Dest("null"), "000");
    EXPECT_EQ(Asm::Dest("XYZ"), "000");
}

TEST(CodeTest, JumpKnown)
{
    EXPECT_EQ(Asm::Jump("JMP"), "111");
    EXPECT_EQ(Asm::Jump("JGT"), "001");
    EXPECT_EQ(Asm::Jump("null"), "000");
    EXPECT_EQ(Asm::Jump("XYZ"), "000");
}

TEST(CodeTest, CompKnown)
{
    // comp codes are 6-bit strings (per implementation)
    EXPECT_EQ(Asm::Comp("A"), std::string("110000"));
    EXPECT_EQ(Asm::Comp("D"), std::string("001100"));
    EXPECT_EQ(Asm::Comp("1"), std::string("111111"));
    EXPECT_EQ(Asm::Comp("0"), std::string("101010"));
    EXPECT_EQ(Asm::Comp("M"), std::string("110000"));
    // unknown comp returns default 6 zeros
    EXPECT_EQ(Asm::Comp("XYZ"), std::string("000000"));
}
