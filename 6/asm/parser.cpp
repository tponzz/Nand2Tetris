#include "parser.h"
#include <algorithm>
#include <cctype>
#include <iostream>
#include <ranges>
#include <string_view>

namespace Asm {

// Must ensure line is a C-instruction
Parser::CInstrunction
Extract(const std::string& line)
{
    Parser::CInstrunction c_inst;

    const size_t pos_eq = line.find('=');
    const size_t pos_scol = line.find(';');

    if (pos_eq != std::string::npos) {
        c_inst.d = line.substr(0, pos_eq);
    }

    if (pos_scol != std::string::npos) {
        c_inst.j = line.substr(pos_scol + 1);
    }

    if (!c_inst.d.empty() && !c_inst.j.empty()) {
        // dest=comp;jump -> comp is between '=' and ';'
        c_inst.c = line.substr(pos_eq + 1, pos_scol - pos_eq - 1);
    } else if (!c_inst.d.empty()) {
        // dest=comp
        c_inst.c = line.substr(pos_eq + 1);
    } else if (!c_inst.j.empty()) {
        // comp;jump
        c_inst.c = line.substr(0, pos_scol);
    }

    return c_inst;
}

static bool
IsLInstruction(const std::string& line)
{
    bool ok = (line.front() == '(') && (line.back() == ')');
    constexpr char words[] = "1234567890abcdefghijklmnopqrstuvwxyz_.$:";
    if (ok) {
        for (auto&& c : line.substr(1, line.size() - 2)) {
            const char lc = std::tolower(c);
            ok &= std::binary_search(std::begin(words), std::end(words), lc);
        }
    }

    return ok;
}

static bool
IsAInstruction(const std::string& line)
{
    bool ok = line.front() == '@';
    constexpr char nums[] = "1234567890";
    if (ok) {
        for (auto&& c : line.substr(1)) {
            ok &= std::binary_search(std::begin(nums), std::end(nums), c);
        }
    }

    return ok;
}

static bool
IsCInstruction(const std::string& line)
{
    constexpr std::string_view jumps[] = {
        "JGT", "JEQ", "JGE", "JLT", "JNE", "JLE", "JMP"
    };
    constexpr std::string_view dests[] = { "M", "D", "DM", "A", "AM", "AD", "ADM" };
    constexpr std::string_view comps[] = { "0",   "1",   "-1",  "D",   "A",   "!A",  "!D",
                                           "-D",  "-A",  "D+1", "A+1", "D-1", "A-1", "D+A",
                                           "D-A", "A-D", "D&A", "D|A", "M",   "!M",  "M+1",
                                           "M-1", "D+M", "D-M", "M-D", "D&M", "D|M" };

    const auto c_inst = Extract(line);

    // helper: check membership
    auto contains = [](auto& arr, const std::string& val) {
        return std::ranges::find(arr, val) != std::end(arr);
    };

    // comp must always be present and valid
    if (c_inst.c.empty() || !contains(comps, c_inst.c)) {
        return false;
    }

    if (!c_inst.d.empty() && !contains(dests, c_inst.d)) {
        return false;
    }

    if (!c_inst.j.empty() && !contains(jumps, c_inst.j)) {
        return false;
    }

    return true;
}

Parser::Parser(const std::string& filepath)
{
    in_.open(filepath);
    if (!in_) {
        std::cerr << "Failed to open file: " << filepath << std::endl;
    }
}

Parser::~Parser()
{
    in_.close();
}

bool
Parser::HasMoreLines()
{
    return !(in_.peek() == std::char_traits<char>::eof());
}

void
Parser::Advance()
{
    while (HasMoreLines()) {
        std::string line;
        std::getline(in_, line);

        if (line.substr(2) == "//") {
            continue;
        }

        cur_ = line;
        break;
    }
}

Parser::Instruction
Parser::InstructionType() const
{
    if (IsLInstruction(cur_)) {
        return Instruction::L;
    }

    if (IsAInstruction(cur_)) {
        return Instruction::A;
    }

    if (IsCInstruction(cur_)) {
        return Instruction::C;
    }

    return Instruction::Invalid;
}

std::string
Parser::Symbol() const
{
    if (IsLInstruction(cur_)) {
        return cur_.substr(1, cur_.size() - 2);
    }

    return std::string();
}

std::string
Parser::Dest() const
{
    if (IsCInstruction(cur_)) {
        const CInstrunction c_inst = Extract(cur_);
        return c_inst.d;
    }

    return std::string();
}

std::string
Parser::Comp() const
{
    if (IsCInstruction(cur_)) {
        const CInstrunction c_inst = Extract(cur_);
        return c_inst.c;
    }

    return std::string();
}

std::string
Parser::Jump() const
{
    if (IsCInstruction(cur_)) {
        const CInstrunction c_inst = Extract(cur_);
        return c_inst.j;
    }

    return std::string();
}

std::string
Parser::Current() const
{
    return cur_;
}

} // namespace Asm