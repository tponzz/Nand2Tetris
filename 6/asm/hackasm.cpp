// Simple example program for CMake demonstration
#include <algorithm>
#include <bitset>
#include <fstream>
#include <iostream>
#include <sstream>

#include "code.h"
#include "parser.h"
#include "symbol_table.h"

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

bool
IsNumber(const std::string& s)
{
    return std::ranges::all_of(s, [](const char c) { return std::isdigit(c) != 0; });
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
    const std::string out_path = argv[2];
    Writer w{ out_path };

    SymbolTable tbl;
    size_t next_addr = 16;

    // 1st path
    {
        Asm::Parser p{ in_path };
        for (size_t nol = 0; p.HasMoreLines();) {
            p.Advance();

            switch (p.InstructionType()) {
                case Asm::Parser::Instruction::L: {
                    const std::string symbol = p.Symbol();
                    if (!tbl.Contains(symbol)) {
                        tbl.AddEntry(symbol, nol);
                    }
                } break;

                case Asm::Parser::Instruction::C:
                case Asm::Parser::Instruction::A: {
                    // L以外の時増やす
                    nol++;
                } break;

                default:
                    break;
            }
        }
    }

    // 2nd path
    Asm::Parser p{ in_path };
    while (p.HasMoreLines()) {
        // 1行読み取り
        p.Advance();

        // 変換
        std::stringstream line;
        switch (p.InstructionType()) {
            case Asm::Parser::Instruction::A: {
                line << "0";
                const std::string symbol = p.Symbol();
                int addr{ 0 };
                if (IsNumber(symbol)) {
                    addr = std::stoi(symbol);
                } else {
                    if (!tbl.Contains(symbol)) {
                        tbl.AddEntry(symbol, next_addr);
                        next_addr++;
                    }

                    addr = tbl.GetAddress(symbol);
                }

                line << itob(addr);
            } break;

            case Asm::Parser::Instruction::C: {
                const auto d = p.Dest();
                const auto c = p.Comp();
                const auto j = p.Jump();

                line << "111";
                line << (c.find('M') != std::string::npos ? "1" : "0");
                line << Asm::Comp(c) << Asm::Dest(d) << Asm::Jump(j);
            } break;

            case Asm::Parser::Instruction::L:
            default:
                break;
        }

        // 書き込み
        w.WriteNextLine(line.str());
    }

    return 0;
}
