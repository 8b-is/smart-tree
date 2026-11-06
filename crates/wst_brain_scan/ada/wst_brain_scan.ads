--  WST_Brain_Scan — Ada 2022 lane for the bent balanced-ternary diff.
--  Part of the 8b.is Weight-Space-Time verification deck.
--
--  References: Ada 2022 Reference Manual (ISO/IEC 8652:2023):
--    3.2   Types and Subtypes
--    3.5.4 Integer Types
--    3.6   Array Types
--    3.8   Record Types
--    4.4   Expressions
--    4.5   Operators and Expression Evaluation
--    5.2   Assignment Statements
--    5.3   If Statements
--    5.4   Case Statements
--    5.5   Loop Statements
--    6.1   Subprogram Declarations
--    6.3   Subprogram Bodies
--    7.3   Package Specifications
--    7.4   Package Bodies

package WST_Brain_Scan is
   pragma Pure;

   type Bal_Trit is range -1 .. 1;

   type Trit_Array is array (Natural range <>) of Bal_Trit;

   function Rotate_Cyclic
     (T : Bal_Trit; Steps_Mod_3 : Natural) return Bal_Trit
     with Inline;

   function Ternary_NAND (A, B : Bal_Trit) return Bal_Trit
     with Inline;

   function Extract_Unbent_Balanced_Trits
     (Seed : Long_Long_Integer; N : Natural) return Trit_Array;

   function Bend_Stream_ASM_Recurrence
     (Unbent : Trit_Array; K_Step : Natural) return Trit_Array;

   type Gate_Label is (Identity, Z_Cycle, X_Cycle);

   type One_Hot_Qubit_Pair is record
      Q1_Neg_Bit : Natural range 0 .. 1 := 0;
      Q0_Pos_Bit : Natural range 0 .. 1 := 0;
      Gate_Applied : Gate_Label := Identity;
   end record;

   function Apply_Bent_Circuit
     (Unbent : Bal_Trit; Bend_Step_Mod3 : Natural) return One_Hot_Qubit_Pair;

   function Measure_Z_Basis (Pair : One_Hot_Qubit_Pair) return Bal_Trit;

   function Verify_Bent_Ternary
     (Seed : Long_Long_Integer; Bend_K : Natural; Num_Trits : Natural)
      return Boolean;

end WST_Brain_Scan;
