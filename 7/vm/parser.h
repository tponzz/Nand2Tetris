#include <fstream>
#include <string>

namespace Vm {

class Parser
{
  public:
    enum class Cmd
    {
        Arithmetic,
        Push,
        Pop,
        Label,
        Goto,
        If,
        Function,
        Return,
        Call,
        Invalid
    };

    Parser(const std::string& file);
    ~Parser();

    // 次の行があるか
    // 開始は1行目の前から
    bool HasMoreLines();

    // 次の行に移動
    void Advance();

    // 現在のコマンドタイプ
    Cmd CommandType() const;

    // コマンドの初めの引数
    // Arithmeticの場合、add/subなど
    std::string Arg1() const;
    int Arg2() const;

  private:
    std::string _cur;
    std::ifstream _in;
};

}