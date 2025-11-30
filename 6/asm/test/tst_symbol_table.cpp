// Unit tests for SymbolTable
#include <gtest/gtest.h>
#include <limits>

#include "../symbol_table.h"

TEST(SymbolTableTest, DefaultMissing)
{
    SymbolTable tbl;
    EXPECT_FALSE(tbl.Contains("FOO"));
    EXPECT_EQ(tbl.GetAddress("FOO"), std::numeric_limits<size_t>::max());

    // predefined symbols should exist
    EXPECT_TRUE(tbl.Contains("SP"));
    EXPECT_EQ(tbl.GetAddress("SP"), 0u);
    EXPECT_TRUE(tbl.Contains("R0"));
    EXPECT_EQ(tbl.GetAddress("R0"), 0u);
    EXPECT_TRUE(tbl.Contains("R15"));
    EXPECT_EQ(tbl.GetAddress("R15"), 15u);
    EXPECT_TRUE(tbl.Contains("SCREEN"));
    EXPECT_EQ(tbl.GetAddress("SCREEN"), 16384u);
    EXPECT_TRUE(tbl.Contains("KBD"));
    EXPECT_EQ(tbl.GetAddress("KBD"), 24576u);
}

TEST(SymbolTableTest, AddAndLookup)
{
    SymbolTable tbl;
    tbl.AddEntry("LOOP", 16);

    EXPECT_TRUE(tbl.Contains("LOOP"));
    EXPECT_EQ(tbl.GetAddress("LOOP"), 16u);

    // Adding again with a different address should NOT overwrite existing entry
    tbl.AddEntry("LOOP", 42);
    EXPECT_EQ(tbl.GetAddress("LOOP"), 16u);
}

TEST(SymbolTableTest, MultipleEntriesIndependent)
{
    SymbolTable tbl;
    tbl.AddEntry("A", 1);
    tbl.AddEntry("B", 2);
    tbl.AddEntry("C", 3);

    EXPECT_EQ(tbl.GetAddress("A"), 1u);
    EXPECT_EQ(tbl.GetAddress("B"), 2u);
    EXPECT_EQ(tbl.GetAddress("C"), 3u);

    EXPECT_TRUE(tbl.Contains("A"));
    EXPECT_TRUE(tbl.Contains("B"));
    EXPECT_TRUE(tbl.Contains("C"));
}
