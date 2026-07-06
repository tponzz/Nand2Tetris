#include "code_writer.h"
#include "parser.h"

#include <cstddef>
#include <cstdlib>
#include <filesystem>
#include <format>
#include <ios>
#include <iostream>
#include <map>
#include <memory>
#include <string>
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

    // SPの指すアドレスがスタックの先頭 or スタックの先頭の次？
    static void Pop(std::ostream& out);
    static void Push(std::ostream& out);
};

void
RamAccessGenerator::Pop(std::ostream& out)
{
    out << "@SP\n"
           "AM=M-1\n"
           "D=M\n";
}

void
RamAccessGenerator::Push(std::ostream& out)
{
    out << "@SP\n"
           "A=M\n"
           "M=D\n"
           "@SP\n"
           "M=M+1\n";
}

class StandardSegGenerator : public RamAccessGenerator
{
  public:
    StandardSegGenerator(const std::string& seg)
      : _seg(seg) {};

    virtual void WritePush(std::ofstream& out, std::size_t idx) override
    {
        // push argument 2
        // push ARG[2] to stack
        out << "@" << idx << "\n"
            << "D=A\n"
            << "@" + _seg + "\n"
            << "A=D+M\n"
               "D=M\n";

        Push(out);
    }

    virtual void WritePop(std::ofstream& out, std::size_t idx) override
    {
        // pop argument 2
        // stack to ARG[2]
        Pop(out); // D = *SP
        out << "@" << _seg << "\n"
            << "A=M\n";
        while (idx--) {
            out << "A=A+1\n";
        }
        out << "M=D\n";
    }

  private:
    std::string _seg; // argument, local, this, that
};

class ConstantGenerator : public RamAccessGenerator
{
  public:
    virtual void WritePush(std::ofstream& out, std::size_t idx) override
    {
        // push constant 22
        out << "@" << idx << "\n"
            << "D=A\n";
        Push(out);
    }

    virtual void WritePop(std::ofstream& out, std::size_t idx) override
    {
        (void)out;
        (void)idx;
    }
};

class StaticGenerator : public RamAccessGenerator
{
    static constexpr int ADDR = 16;

  public:
    virtual void WritePush(std::ofstream& out, std::size_t idx) override
    {
        out << "@" << (ADDR + idx) << "\n"
            << "D=M\n";
        Push(out);
    };

    virtual void WritePop(std::ofstream& out, std::size_t idx) override
    {
        Pop(out);
        out << "@" << (ADDR + idx) << "\n"
            << "M=D\n";
    }
};

class PointerGenerator : public RamAccessGenerator
{
    static constexpr std::string_view SEG[2] = { "THIS", "THAT" };

  public:
    virtual void WritePush(std::ofstream& out, std::size_t idx) override
    {
        out << "@" << SEG[idx] << "\n"
            << "D=M\n";
        Push(out);
    }

    virtual void WritePop(std::ofstream& out, std::size_t idx) override
    {
        // pop this 6
        // pop that 2
        Pop(out);
        out << "@" << SEG[idx] << "\n"
            << "M=D\n";
    }
};

class TempGenerator : public RamAccessGenerator
{
    constexpr static int BASE = 5;

  public:
    virtual void WritePush(std::ofstream& out, std::size_t idx) override
    {
        // push temp 6
        out << "@" << (idx + BASE) << "\n"
            << "D=M\n";
        Push(out);
    }

    virtual void WritePop(std::ofstream& out, std::size_t idx) override
    {
        // pop temp 6
        Pop(out);
        out << "@" << (idx + BASE) << "\n"
            << "M=D\n";
    }
};

static const std::map<std::string, std::shared_ptr<RamAccessGenerator>> STACK_GENS{
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
    void Pop2DReg(std::ostream& out)
    {
        out << "@SP\n"
               "AM=M-1\n"
               "D=M\n";
    }

    void Sub(std::ostream& out)
    {
        out << "@SP\n"
               "AM=M-1\n"
               "D=M-D\n";
    }

    void PushFalse(std::ostream& out)
    {
        out << "@SP\n"
               "A=M\n"
               "M=0\n";
    }

    void PushTrue(const std::string& jump, const int id, std::ostream& out)
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
               "A=M\n"
               "M=-1\n";

        out << end_label << "\n"
            << "@SP\n"
               "M=M+1\n";
    }
};

// add  : x+y
class AddGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        Pop2DReg(out);
        out << "@SP\n"
               "A=M-1\n"
               "M=D+M\n";
    };
};

// sub  : x-y
class SubGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        Pop2DReg(out);
        out << "@SP\n"
               "A=M-1\n"
               "M=M-D\n";
    };
};

// neg  : -y
class NegGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        Pop2DReg(out);
        out << "@SP\n"
               "A=M\n"
               "M=-D\n"
               "@SP\n"
               "M=M+1\n";
    };
};

// eq   : x == y
class EqGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        static int id = 0;

        // pop y into D
        Pop2DReg(out);
        // pop x and compute D = x - y
        Sub(out);
        // default: push false (0)
        PushFalse(out);
        // if equal, set true (-1)
        PushTrue("JEQ", id++, out);
    };
};

// gt   : x > y
class GtGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        static int id = 0;

        // pop y into D
        Pop2DReg(out);
        // pop x and compute D = x - y
        Sub(out);
        // default: push false (0)
        PushFalse(out);
        // if greater, set true (-1)
        PushTrue("JGT", id++, out);
    };
};

// lt   : x < y
class LtGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        static int id = 0;

        // pop y into D
        Pop2DReg(out);
        // pop x and compute D = x - y
        Sub(out);
        // default: push false (0)
        PushFalse(out);
        // if less, set true (-1)
        PushTrue("JLT", id++, out);
    };
};

// and  : x & y
class AndGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        Pop2DReg(out);
        out << "@SP\n"
               "A=M-1\n"
               "M=D&M\n";
    };
};

// or   : x | y
class OrGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        Pop2DReg(out);
        out << "@SP\n"
               "A=M-1\n"
               "M=D|M\n";
    };
};

// not  : !y
class NotGenerator : public ArithmeticGenerator
{
    virtual void WriteArithmetic(std::ofstream& out) final
    {
        Pop2DReg(out);
        out << "@SP\n"
               "A=M\n"
               "M=!D\n"
               "@SP\n"
               "M=M+1\n";
    };
};

static const std::map<std::string, std::shared_ptr<ArithmeticGenerator>> ARITH_GENS{
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
    // open output file
    try {
        _out.open(filepath, std::ios::out | std::ios::trunc);
    } catch (const std::exception& e) {
        std::cerr << e.what() << '\n';
        std::exit(1);
    }

    namespace fs = std::filesystem;
    this->SetFileName(fs::path(filepath).stem());

    // boot strap code
    _out << "@256\n"
            "D=A\n"
            "@SP\n"
            "M=D\n";

    // call sys.init
    WriteCall("Sys.init", 0);
}

CodeWriter::~CodeWriter() {}

void
CodeWriter::SetFileName(const std::string& filename)
{
    this->_filename = filename;
}

void
CodeWriter::WriteArithmetic(const std::string& cmd_line)
{
    auto&& gen = ARITH_GENS.at(cmd_line);
    gen->WriteArithmetic(_out);
}

void
CodeWriter::WritePushPop(Parser::Cmd cmd, const std::string& seg, const size_t idx)
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

        case Parser::Cmd::Call:
        case Parser::Cmd::Function:
        case Parser::Cmd::Return:
        case Parser::Cmd::Arithmetic:
        case Parser::Cmd::Goto:
        case Parser::Cmd::If:
        case Parser::Cmd::Label:
        case Parser::Cmd::Invalid:
            break;
    }
}

void
CodeWriter::WriteLabel(const std::string& label)
{
    _out << std::format("({})\n", label);
}

void
CodeWriter::WriteGoto(const std::string& label)
{
    _out << '@' << label << '\n' << "0;JMP\n";
}

void
CodeWriter::WriteIf(const std::string& label)
{

    _out << "@SP\n"
            "AM=M-1\n"
            "D=M\n"
         << "@" << label << '\n'
         << "D;JNE\n";
}

void
CodeWriter::WriteFuntion(const std::string& function_name, const int n_vars)
{
    _out << std::format("({})\n", function_name);
    for (int i = 0; i < n_vars; i++) {
        _out << "@SP\n"
                "A=M\n"
                "M=0\n"
                "@SP\n"
                "M=M+1\n";
    }
}

std::string
MakeReturnSymbol(const std::string& filename, const std::string& label, const int idx)
{
    return std::format("{}.{}$ret.{}", filename, label, idx);
}

void
CodeWriter::WriteCall(const std::string& function_name, const int n_vars)
{
    // call count in runtime
    static int idx = 0;

    // push return address
    const auto symbol = MakeReturnSymbol(_filename, function_name, idx);
    _out << "@" << symbol << "\n"
         << "D=A\n";
    RamAccessGenerator::Push(_out);

    // push LCL ARG THIS THAT
    // 親の値を保存しておく
    auto push_label = [](std::ofstream& out, const std::string& label) {
        out << "@" << label << "\n"
            << "D=M\n";
        RamAccessGenerator::Push(out);
    };

    push_label(_out, "LCL");
    push_label(_out, "ARG");
    push_label(_out, "THIS");
    push_label(_out, "THAT");

    // 関数内のデータに上書き
    // ARG = SP-5-nArgs
    _out << "@" << 5 + n_vars
         << "\n"
            "D=A\n"
            "@SP\n"
            "D=M-D\n"
            "@ARG\n"
            "M=D\n";

    // LCL = SP
    _out << "@SP\n"
            "D=M\n"
            "@LCL\n"
            "M=D\n";

    // goto f
    this->WriteGoto(function_name);

    // (return symbol)
    _out << '(' << symbol << ")\n";

    // Increment for next call
    idx++;
}

void
CodeWriter::WriteReturn()
{
    // ★ 定数を引きたいときはアドレス値を使う
    //   @5
    //   D=D-A (=> D-=5)
    // ★ R13-R15 VM変換器の生成コードに変数が必要な場合、これらのレジスタを使用可能（本書p.175）

    // LCL = HEAD = SP
    // frame: R13 = LCL
    _out << "@LCL\n"
            "D=M\n"
            "@R13\n"
            "M=D\n";

    // retAddr: R14 = *(frame-5)
    _out << "@5\n"
            "A=D-A\n"
            "D=M\n"
            "@R14\n"
            "M=D\n";

    // D=pop()
    RamAccessGenerator::Pop(_out);

    // *ARG=pop() (= *ARG=D)
    _out << "@ARG\n"
            "A=M\n"
            "M=D\n";

    // SP = ARG+1
    _out << "@ARG\n"
            "D=M+1\n"
            "@SP\n"
            "M=D\n";

    // THAT=*(frame-1)
    // THIS=*(frame-2)
    // ARG=*(frame-3)
    // LCL=*(frame-4)
    auto deref = []() {
        return "@R13\n"
               "AM=M-1\n" // D=frame-1; @R13=frame-1
               "D=M";     // D=*(frame-offset)
    };

    _out << deref() << "\n"
         << "@THAT\n"
            "M=D\n"; // THAT=*(frame-1)
    _out << deref() << "\n"
         << "@THIS\n"
            "M=D\n"; // THAT=*(frame-2)
    _out << deref() << "\n"
         << "@ARG\n"
            "M=D\n"; // THAT=*(frame-3)
    _out << deref() << "\n"
         << "@LCL\n"
            "M=D\n"; // THAT=*(frame-4)

    // goto retAddr
    _out << "@R14\n"
            "A=M\n"
            "0;JMP\n";
}

void
CodeWriter::Close()
{
    if (_out.is_open()) {
        _out.close();
    }
}

} // namespace Vm
