#include "code_writer.h"
#include <format>
#include <iostream>
#include <map>
#include <memory>
#include <optional>
#include <sstream>
#include <string>
#include <vector>

namespace Vm {

// スタック(RAM[256-2047])とセグメント(RAM[0-255])は別
// Popはスタックの先頭からセグメントに値をコピー
// Pushはスタックの先頭にセグメントから値をコピー

// | SP |

class RamAccessGenerator
{
  public:
    virtual ~RamAccessGenerator()                               = default;
    virtual void WritePush(std::ofstream& out, std::size_t idx) = 0;
    virtual void WritePop(std::ofstream& out, std::size_t idx)  = 0;

  protected:
    // SPの指すアドレスがスタックの先頭 or スタックの先頭の次？
    static void Pop(std::stringstream& ss);
    static void Push(std::stringstream& ss);
};

void
RamAccessGenerator::Pop(std::stringstream& ss)
{
    ss << "@SP\n"
       << "M=M-1\n"
       << "A=M\n"
       << "D=M\n";
}

void
RamAccessGenerator::Push(std::stringstream& ss)
{
    ss << "@SP\n"
       << "A=M\n"
       << "M=D\n"
       << "@SP\n"
       << "M=M+1\n";
}

class StandardSegGenerator : public RamAccessGenerator
{
  public:
    StandardSegGenerator(const std::string& seg)
      : _seg(seg) {};

    virtual void WritePush(std::ofstream& out, std::size_t idx) override
    {
        // push argument 2
        std::stringstream ss;
        ss << "@" + _seg + "\n";
        ss << "D=M\n";
        Push(ss);
        out << ss.str();
    }

    virtual void WritePop(std::ofstream& out, std::size_t idx) override
    {
        // pop argument 2
        std::stringstream ss;
        Pop(ss);
        ss << "@" << _seg << "\n";
        ss << "M=D\n";
        out << ss.str();
    }

  private:
    std::string _seg;
};

class ConstantGenerator : public RamAccessGenerator
{
  public:
    virtual void WritePush(std::ofstream& out, std::size_t idx) override
    {
        // push constant 22
        std::stringstream ss;
        ss << "@" << std::to_string(idx) << "\n"
           << "D=A\n";
        Push(ss);
        out << ss.str();
    }

    virtual void WritePop(std::ofstream& out, std::size_t idx) override {}
};

class StaticGenerator : public RamAccessGenerator
{
    static constexpr int ADDR = 16;

  public:
    virtual void WritePush(std::ofstream& out, std::size_t idx) override
    {
        std::stringstream ss;
        ss << "@" << std::to_string(ADDR + idx) << "\n"
           << "D=M\n";
        Push(ss);
        out << ss.str();
    };

    virtual void WritePop(std::ofstream& out, std::size_t idx) override
    {
        std::stringstream ss;
        Pop(ss);
        ss << "@" << std::to_string(ADDR + idx) << "\n"
           << "M=D\n";
        out << ss.str();
    }
};

class PointerGenerator : public RamAccessGenerator
{
    static constexpr std::string_view SEG[2] = { "THIS", "THAT" };

  public:
    virtual void WritePush(std::ofstream& out, std::size_t idx) override
    {
        std::stringstream ss;
        ss << "@" << SEG[idx] << "\n";
        ss << "D=M\n";
        Push(ss);
        out << ss.str();
    }

    virtual void WritePop(std::ofstream& out, std::size_t idx) override
    {
        // pop this 6
        // pop that 2
        std::stringstream ss;
        Pop(ss);
        ss << "@" << SEG[idx] << "\n";
        ss << "M=D\n";
        out << ss.str();
    }
};

class TempGenerator : public RamAccessGenerator
{
    constexpr static int BASE = 5;

  public:
    virtual void WritePush(std::ofstream& out, std::size_t idx) override
    {
        // push temp 6
        std::stringstream ss;
        ss << "@" << std::to_string(idx + BASE) << "\n"
           << "D=M\n";
        Push(ss);
        out << ss.str();
    }

    virtual void WritePop(std::ofstream& out, std::size_t idx) override
    {
        // pop temp 6
        std::stringstream ss;
        Pop(ss);
        ss << "@" << std::to_string(idx + BASE) << "\n"
           << "M=D\n";
        out << ss.str();
    }
};

static const std::map<std::string, std::shared_ptr<RamAccessGenerator>>
  STACK_GENS{
      { "argument",  std::make_shared<StandardSegGenerator>("ARG") },
      {    "local",  std::make_shared<StandardSegGenerator>("LCL") },
      {     "this", std::make_shared<StandardSegGenerator>("THIS") },
      {     "that", std::make_shared<StandardSegGenerator>("THAT") },
      { "constant",          std::make_shared<ConstantGenerator>() },
      {   "static",            std::make_shared<StaticGenerator>() },
      {  "pointer",           std::make_shared<PointerGenerator>() },
      {     "temp",              std::make_shared<TempGenerator>() },
};

class ArithmeticGenerator
{
  public:
    ~ArithmeticGenerator()                           = default;
    virtual void WriteArithmetic(std::ofstream& out) = 0;

  protected:
    void Pop2DReg(std::stringstream& out)
    {
        out << "@SP\n"
            << "M=M-1\n"
            << "A=M\n"
            << "D=M\n";
    }

    void Sub(std::stringstream& out)
    {
        out << "@SP\n"
            << "M=M-1\n"
            << "A=M\n"
            << "D=M-D\n";
    }

    void PushFalse(std::stringstream& out)
    {
        out << "@SP\n"
            << "A=M\n"
            << "M=0\n";
    }

    void PushTrue(const std::string& jump, const int id, std::stringstream& out)
    {
        auto&& true_label      = std::format("({}_{}_{})", jump, "TRUE", id);
        auto&& end_label       = std::format("({}_{}_{})", jump, "END", id);
        auto&& call_true_label = std::format("@{}_{}_{}", jump, "TRUE", id);
        auto&& call_end_label  = std::format("@{}_{}_{}", jump, "END", id);

        out << call_true_label << "\n"
            << "D;" + jump << "\n"
            << call_end_label << "\n"
            << "0;JMP\n";

        out << true_label << "\n"
            << "@SP\n"
            << "A=M\n"
            << "M=-1\n";

        out << end_label << "\n"
            << "@SP\n"
            << "M=M+1\n";
    }
};

// add  : x+y
class AddGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        std::stringstream ss;
        Pop2DReg(ss);
        ss << "@SP\n"
           << "A=M-1\n"
           << "M=D+M\n";
        out << ss.str();
    };
};

// sub  : x-y
class SubGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        std::stringstream ss;
        Pop2DReg(ss);
        ss << "@SP\n"
           << "A=M-1\n"
           << "M=D-M\n";
        out << ss.str();
    };
};

// neg  : -y
class NegGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        std::stringstream ss;
        Pop2DReg(ss);
        ss << "@SP\n"
           << "A=M\n"
           << "M=-D\n"
           << "@SP\n"
           << "M=M+1\n";
        out << ss.str();
    };
};

// eq   : x == y
class EqGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        static int id = 0;

        std::stringstream ss;
        // pop y into D
        Pop2DReg(ss);
        // pop x and compute D = x - y
        Sub(ss);
        // default: push false (0)
        PushFalse(ss);
        // if equal, set true (-1)
        PushTrue("JEQ", id++, ss);

        out << ss.str();
    };
};

// gt   : x > y
class GtGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        static int id = 0;

        std::stringstream ss;
        // pop y into D
        Pop2DReg(ss);
        // pop x and compute D = x - y
        Sub(ss);
        // default: push false (0)
        PushFalse(ss);
        // if greater, set true (-1)
        PushTrue("JGT", id++, ss);

        out << ss.str();
    };
};

// lt   : x < y
class LtGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        static int id = 0;

        std::stringstream ss;
        // pop y into D
        Pop2DReg(ss);
        // pop x and compute D = x - y
        Sub(ss);
        // default: push false (0)
        PushFalse(ss);
        // if less, set true (-1)
        PushTrue("JLT", id++, ss);

        out << ss.str();
    };
};

// and  : x & y
class AndGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        std::stringstream ss;
        Pop2DReg(ss);
        ss << "@SP\n"
           << "A=M-1\n"
           << "M=D&M\n";
        out << ss.str();
    };
};

// or   : x | y
class OrGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        std::stringstream ss;
        Pop2DReg(ss);
        ss << "@SP\n"
           << "A=M-1\n"
           << "M=D|M\n";
        out << ss.str();
    };
};

// not  : !y
class NotGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        std::stringstream ss;
        Pop2DReg(ss);
        ss << "@SP\n"
           << "A=M\n"
           << "M=!D\n"
           << "@SP\n"
           << "M=M+1\n";
        out << ss.str();
    };
};

static const std::map<std::string, std::shared_ptr<ArithmeticGenerator>>
  ARITH_GENS{
      { "add", std::make_shared<AddGenerator>() },
      { "sub", std::make_shared<SubGenerator>() },
      { "neg", std::make_shared<NegGenerator>() },
      {  "eq",  std::make_shared<EqGenerator>() },
      {  "gt",  std::make_shared<GtGenerator>() },
      {  "lt",  std::make_shared<LtGenerator>() },
      { "and", std::make_shared<AndGenerator>() },
      {  "or",  std::make_shared<OrGenerator>() },
      { "not", std::make_shared<NotGenerator>() },
};

CodeWriter::CodeWriter(const std::string& filepath)
{
    try {
        _out.open(filepath);
    } catch (const std::exception& e) {
        std::cerr << e.what() << '\n';
    }
}

CodeWriter::~CodeWriter() {}

void
CodeWriter::WriteArithmetic(const std::string& cmd_line)
{
    auto&& gen = ARITH_GENS.at(cmd_line);
    gen->WriteArithmetic(_out);
}

void
CodeWriter::WritePushPop(Parser::Cmd cmd,
                         const std::string& seg,
                         const size_t idx)
{
    switch (cmd) {
        case Parser::Cmd::Push: {
            auto&& gen = STACK_GENS.at(seg);
            gen->WritePush(_out, idx);
        } break;

        case Parser::Cmd::Pop: {
            auto&& gen = STACK_GENS.at(seg);
            gen->WritePop(_out, idx);
        } break;

        default:
            break;
    }
}

void
CodeWriter::Close()
{
    if (_out.is_open()) {
        _out.close();
    }
}

} // namespace Vm