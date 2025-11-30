// Simple example program for CMake demonstration
#include <bitset>
#include <fstream>
#include <iostream>
#include <sstream>

#include "code.h"
#include "parser.h"

class Writer
{
  private:
    std::ofstream file_;

  public:
    Writer(const std::string& path)
    {
        file_.open(path, std::ios::out | std::ios::trunc);
        if (!file_.is_open()) {
            std::cerr << "Failed to open the file(" << path << ")\n";
        }
    }
    ~Writer()
    {
        if (file_.is_open())
            file_.close();
    }

    void WriteNextLine(const std::string& content)
    {
        if (!content.empty()) {
            std::cout << "Write: " << content << std::endl;
            file_ << content << std::endl;
        }
    }
};

std::string
itob(const size_t addr)
{
    std::bitset<15> bits{ addr };
    return bits.to_string();
}

void
Usage()
{
    std::cout << "Usage: hackasm <in-path> <out-path>";
    std::cout << "  in-path  : path to .asm file";
    std::cout << "  out-path : path to .bin file";
}

int
main(int argc, char** argv)
{
    if (argc != 3) {
        Usage();
        return -1;
    }

    const std::string in_path = argv[1];
    Asm::Parser p{ in_path };

    const std::string out_path = argv[2];
    Writer w{ out_path };

    while (p.HasMoreLines()) {
        // 1行読み取り
        p.Advance();

        // 変換
        std::stringstream line;
        if (p.InstructionType() == Asm::Parser::Instruction::A) {
            line << "0";
            const int addr = std::stoi(p.Current().substr(1));
            line << itob(addr);
        }

        if (p.InstructionType() == Asm::Parser::Instruction::C) {
            const auto d = Asm::Dest(p.Dest());
            const auto c = Asm::Comp(p.Comp());
            const auto j = Asm::Jump(p.Jump());

            line << "111";
            line << (c.find('M') != std::string::npos ? "1" : "0");
            line << d << c << j;
        }

        // 書き込み
        w.WriteNextLine(line.str());
    }

    return 0;
}
