#include "parser.h"

#include <algorithm>
#include <iostream>
#include <ranges>
#include <sstream>
#include <vector>

namespace Vm {

static std::string_view ARITHMETICS[]{ "add", "sub", "neg", "eq", "gt", "lt", "and", "or", "not" };
static std::string_view PUSH{ "push" };
static std::string_view POP{ "pop" };

static std::vector<std::string>
SplitCmd(const std::string& cmd)
{
    std::vector<std::string> ret;
    std::stringstream ss;
    std::string word;

    ss << cmd;
    while (std::getline(ss, word, ' ')) {
        ret.push_back(word);
    }

    return ret;
}

static void
RemoveComments(std::string& s)
{
    const auto comment_beg = s.find_first_of("//");
    if (comment_beg != std::string::npos)
        s.erase(comment_beg);
}

static void
Trim(std::string& s)
{
    // trim left side
    if (!s.empty()) {
        const auto t_space_end = s.find_first_not_of(" \t");
        s.erase(0, t_space_end);
    }

    if (!s.empty()) {
        // trim right side
        const auto b_space_start = s.find_last_not_of(" \t");
        s.erase(b_space_start + 1);
    }
}

Parser::Parser(const std::string& file)
{
    try {
        _in.open(file);
    } catch (const std::exception& e) {
        std::cerr << "[Error] Failed to open the file: ";
        std::cerr << e.what() << '\n';
    }
}

Parser::~Parser()
{
    if (_in) {
        _in.close();
    }
}

bool
Parser::HasMoreLines()
{
    return _in.peek() != std::char_traits<char>::eof();
}

void
Parser::Advance()
{
    constexpr std::string_view delims{ "\n\r" };
    constexpr std::string_view spaces{ " \t" };

    char c;
    std::string line;
    while (HasMoreLines()) {
        line.clear();

        // 改行文字判定（\n と \r の両方に対応）
        while ((c = _in.get()) != std::char_traits<char>::eof()) {
            if (delims.find(c) != std::string::npos) {
                break;
            }

            line += c;
        }

        // インラインコメントを削除
        RemoveComments(line);

        // 先頭と末尾の空白を削除
        Trim(line);

        // 有効な行が見つかったら格納して終了
        if (!line.empty()) {
            _cur = line;
            return;
        }
    }
}

Parser::Cmd
Parser::CommandType() const
{
    std::stringstream ss;
    ss << _cur;

    std::string cmd;
    ss >> cmd;

    if (std::ranges::find(ARITHMETICS, cmd) != std::end(ARITHMETICS)) {
        return Cmd::Arithmetic;
    }

    if (cmd == PUSH) {
        return Cmd::Push;
    }

    if (cmd == POP) {
        return Cmd::Pop;
    }

    return Cmd::Invalid;
}

std::string
Parser::Arg1() const
{
    if (this->CommandType() == Cmd::Arithmetic) {
        const auto cmds = SplitCmd(_cur);
        return cmds.front();
    }

    return std::string();
}

int
Parser::Arg2() const
{
    constexpr Cmd HAS_ARG2[]{ Cmd::Push, Cmd::Pop, Cmd::Function, Cmd::Call };

    if (std::ranges::find(HAS_ARG2, this->CommandType()) != std::end(HAS_ARG2)) {
        const auto cmds = SplitCmd(_cur);
        return std::stoi(cmds.back());
    }

    return -1;
}

} // namespace Vm