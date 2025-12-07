// Parser tests - fixtures and templates
#include <cstdio>
#include <fstream>
#include <gtest/gtest.h>
#include <string>

#include "../parser.h"

using Vm::Parser;

// Test fixture for Parser tests. Creates a temporary file for each test.
class ParserTest : public ::testing::Test
{
  protected:
    std::string tmp_filename;

    void SetUp() override
    {
        // create a unique temp file in /tmp
        tmp_filename = std::string("/tmp/vm_parser_test_") + std::to_string(::getpid()) + "_" +
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
    writeFile("// push x\n// push 7\n// lt\npush 8\neq\nor\n");
    Parser p(tmp_filename);

    EXPECT_TRUE(p.HasMoreLines());
    p.Advance();
    EXPECT_TRUE(p.HasMoreLines());
    p.Advance();
    EXPECT_TRUE(p.HasMoreLines());
    p.Advance();
    EXPECT_FALSE(p.HasMoreLines());
}

// Advance - template to verify advancing to next instruction
TEST_F(ParserTest, Advance)
{
    GTEST_SKIP();
    // writeFile("@0\n@1\n");
    // Parser p(tmp_filename);

    // EXPECT_EQ(p.Current(), "");
    // p.Advance();
    // EXPECT_EQ(p.Current(), "@0");
    // p.Advance();
    // EXPECT_EQ(p.Current(), "@1");
}

// InstructionType - template for classifying instructions
TEST_F(ParserTest, CommandType)
{
    writeFile("// push x\n// push 7\n// lt\npush 8\neq\nor\n");
    Parser p(tmp_filename);

    EXPECT_EQ(p.CommandType(), Parser::Cmd::Invalid);
    p.Advance();
    EXPECT_EQ(p.CommandType(), Parser::Cmd::Push);
    p.Advance();
    EXPECT_EQ(p.CommandType(), Parser::Cmd::Arithmetic);
    p.Advance();
    EXPECT_EQ(p.CommandType(), Parser::Cmd::Arithmetic);
}

// Arg1 - get the first argument when the current line is a Arithmetic
TEST_F(ParserTest, Arg1)
{
    writeFile("// push x\n// push 7\nadd\npush local 8\neq\nor\n");
    Parser p(tmp_filename);

    p.Advance();
    EXPECT_EQ(p.Arg1(), "add");
    p.Advance();
    EXPECT_EQ(p.Arg1(), "local");
}

// Arg2 - get the second argument
TEST_F(ParserTest, Arg2)
{
    writeFile("// push x\n// push 7\nlt\npush local 8\neq\nor\n");
    Parser p(tmp_filename);
    p.Advance();
    EXPECT_EQ(p.Arg2(), -1);
    p.Advance();
    EXPECT_EQ(p.Arg2(), 8);
}

int
main(int argc, char** argv)
{
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
