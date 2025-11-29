// Parser tests - fixtures and templates
#include <cstdio>
#include <fstream>
#include <gtest/gtest.h>
#include <string>

#include "../parser.h"

using Asm::Parser;

// Test fixture for Parser tests. Creates a temporary file for each test.
class ParserTest : public ::testing::Test
{
  protected:
    std::string tmp_filename;

    void SetUp() override
    {
        // create a unique temp file in /tmp
        tmp_filename = std::string("/tmp/asm_parser_test_") + std::to_string(::getpid()) + "_" +
                       std::to_string(::rand());
    }

    void TearDown() override { std::remove(tmp_filename.c_str()); }

    void writeFile(const std::string& contents)
    {
        std::ofstream out(tmp_filename);
        out << contents;
        out.close();
    }
};

// HasMoreLines - template to verify whether parser reports remaining input correctly
TEST_F(ParserTest, HasMoreLines)
{
    writeFile("@0\n@1\n");
    Parser p(tmp_filename);

    EXPECT_TRUE(p.HasMoreLines()) << p.HasMoreLines();
    p.Advance();
    EXPECT_TRUE(p.HasMoreLines()) << p.HasMoreLines();
    p.Advance();
    EXPECT_FALSE(p.HasMoreLines()) << p.HasMoreLines();
}

// Advance - template to verify advancing to next instruction
TEST_F(ParserTest, Advance)
{
    writeFile("@0\n@1\n");
    Parser p(tmp_filename);

    EXPECT_EQ(p.Current(), "");
    p.Advance();
    EXPECT_EQ(p.Current(), "@0");
    p.Advance();
    EXPECT_EQ(p.Current(), "@1");
}

// InstructionType - template for classifying instructions
TEST_F(ParserTest, InstructionType)
{
    writeFile("(LABEL)\n@123\nD=A\n");
    Parser p(tmp_filename);

    EXPECT_EQ(p.InstructionType(), Parser::Instruction::Invalid) << (int)p.InstructionType();
    p.Advance();
    EXPECT_EQ(p.InstructionType(), Parser::Instruction::L) << (int)p.InstructionType();
    p.Advance();
    EXPECT_EQ(p.InstructionType(), Parser::Instruction::A) << (int)p.InstructionType();
    p.Advance();
    EXPECT_EQ(p.InstructionType(), Parser::Instruction::C) << (int)p.InstructionType();
}

// Additional checks for different C-instruction forms
TEST_F(ParserTest, CInstructionWithEquals)
{
    writeFile("D=A\n");
    Parser p(tmp_filename);
    p.Advance();
    EXPECT_EQ(p.InstructionType(), Parser::Instruction::C);
}

TEST_F(ParserTest, CInstructionWithSemicolon)
{
    writeFile("0;JMP\n");
    Parser p(tmp_filename);
    p.Advance();
    EXPECT_EQ(p.InstructionType(), Parser::Instruction::C);
}

TEST_F(ParserTest, CInstructionWithDestCompJump)
{
    writeFile("D=A;JMP\n");
    Parser p(tmp_filename);
    p.Advance();
    EXPECT_EQ(p.InstructionType(), Parser::Instruction::C);
}

// Symbol - template for extracting symbols and numeric values
TEST_F(ParserTest, Symbol)
{
    writeFile("(LOOP)\n@123\n");
    Parser p(tmp_filename);
    p.Advance();
    EXPECT_EQ(p.Symbol(), "LOOP");
    p.Advance();
    EXPECT_TRUE(p.Symbol().empty());
}

// Dest - template for C-instruction destination parsing
TEST_F(ParserTest, Dest)
{
    writeFile("D=A\nM=D\n");
    Parser p(tmp_filename);
    p.Advance();
    EXPECT_EQ(p.Dest(), "D");
    p.Advance();
    EXPECT_EQ(p.Dest(), "M");
}

// Comp - template for C-instruction computation field parsing
TEST_F(ParserTest, Comp)
{
    writeFile("D=A+1\nM=D-1\n");
    Parser p(tmp_filename);
    p.Advance();
    EXPECT_EQ(p.Comp(), "A+1");
    p.Advance();
    EXPECT_EQ(p.Comp(), "D-1");
}

// Jump - template for C-instruction jump parsing
TEST_F(ParserTest, Jump)
{
    writeFile("0;JMP\nD;JGT\n");
    Parser p(tmp_filename);

    p.Advance();
    EXPECT_EQ(p.Jump(), "JMP");
    p.Advance();
    EXPECT_EQ(p.Jump(), "JGT");
}

int
main(int argc, char** argv)
{
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
