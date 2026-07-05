#include "parser.h"

#include <algorithm>
#include <cstddef>
#include <iostream>
#include <ranges>
#include <string>
#include <vector>

namespace Vm {

static std::string_view ARITHMETICS[]{ "add", "sub", "neg", "eq", "gt", "lt", "and", "or", "not" };
static std::string_view PUSH{ "push" };
static std::string_view POP{ "pop" };
static std::string_view FUNCTION{ "function" };
static std::string_view IF{ "if-goto" };
static std::string_view CALL{ "call" };
static std::string_view RETURN{ "return" };
static std::string_view LABEL{ "label" };
static std::string_view GOTO{ "goto" };

static constexpr std::string_view DELIMS{ "\n\r" };
static constexpr std::string_view SPACES{ " \t" };

struct CmdTokens
{
    std::string cmd;
    std::vector<std::string> args;
};

static CmdTokens
SplitCmd(const std::string& cmd)
{
    CmdTokens t{};
    for (const auto token : cmd | std::views::split(' ')) {
        if (t.cmd.empty()) {
            t.cmd = std::string{ token.begin(), token.end() };
        }

        t.args.push_back(std::string{ token.begin(), token.end() });
    }
    return t;
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
        const auto t_space_end = s.find_first_not_of(SPACES);
        s.erase(0, t_space_end);
    }

    if (!s.empty()) {
        // trim right side
        const auto b_space_start = s.find_last_not_of(SPACES);
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

    char c;
    std::string line;
    while (HasMoreLines()) {
        line.clear();

        // 改行文字判定（\n と \r の両方に対応）
        while ((c = _in.get()) != std::char_traits<char>::eof()) {
            if (c == '\r' && _in.peek() == '\n') {
                _in.get();
            }

            if (DELIMS.contains(c)) {
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
    auto tokens = SplitCmd(_cur);

    if (std::ranges::contains(ARITHMETICS, tokens.cmd)) {
        return Cmd::Arithmetic;
    }

    if (tokens.cmd == PUSH) {
        return Cmd::Push;
    }

    if (tokens.cmd == POP) {
        return Cmd::Pop;
    }

    if (tokens.cmd == LABEL) {
        return Cmd::Label;
    }

    if (tokens.cmd == FUNCTION) {
        return Cmd::Function;
    }

    if (tokens.cmd == GOTO) {
        return Cmd::Goto;
    }

    if (tokens.cmd == RETURN) {
        return Cmd::Return;
    }

    if (tokens.cmd == IF) {
        return Cmd::If;
    }

    if (tokens.cmd == CALL) {
        return Cmd::Call;
    }

    return Cmd::Invalid;
}

std::string
Parser::Arg1() const
{
    const Cmd cmd = this->CommandType();
    switch (cmd) {
        case Cmd::Arithmetic: {
            const auto tokens = SplitCmd(_cur);
            return tokens.cmd;
        }
        case Cmd::Push:
        case Cmd::Pop:
        case Cmd::Label:
        case Cmd::Goto:
        case Cmd::If: {
            const auto tokens = SplitCmd(_cur);
            return tokens.args[0];
        }
        case Cmd::Call:
        case Cmd::Return:
        case Cmd::Function:
        case Cmd::Invalid: {
            std::cerr << "Invalid command: " << this->_cur << "\n";
        } break;
        default:
            break;
    }

    return std::string();
}

int
Parser::Arg2() const
{
    const auto tokens = SplitCmd(_cur);
    const size_t len  = tokens.args.size();
    if (len == 2UL) {
        return static_cast<int>(len);
    }

    return -1;
}

} // namespace Vm
