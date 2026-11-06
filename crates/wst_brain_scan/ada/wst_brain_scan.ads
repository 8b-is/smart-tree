--  WST_Brain_Scan - Ada 2022 lane for the bent balanced-ternary diff.
--  Part of the 8b.is Weight-Space-Time verification deck.
--
--  Coding practices follow Perkins, "Programming Practices: Analysis of Ada
--  Source Developed for the Air Force, Army, and Navy" (ACM, 1989): no
--  PRAGMAs, no USE clauses, user-defined integer types, invariant objects
--  declared as CONSTANTS, record aggregates instead of componentwise
--  assignment, FOR loops instead of flag-driven WHILE loops, user-defined
--  EXCEPTIONs, PRIVATE types for encapsulated state, and separate
--  specification/body files.
--
--  References: Ada 2022 Reference Manual (ISO/IEC 8652:2023):
--    3.2   Types and Subtypes
--    3.4   Derived Types and Classes
--    3.5   Scalar Types
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

   type Bal_Trit is range -1 .. 1;

   type Trit_Steps is mod 3;

   type Trit_Array is array (Natural range <>) of Bal_Trit;

   type One_Hot_Qubit_Pair is private;

   --  Exact rational p/q type for weight-space scaling factors.
   --  Denominator is always positive; zero is represented as 0/1.
   type Exact_Ratio is private;

   function Rotate_Cyclic (T : Bal_Trit; Steps : Trit_Steps) return Bal_Trit;

   function Ternary_NAND (A, B : Bal_Trit) return Bal_Trit;

   function Extract_Unbent_Balanced_Trits
     (Seed : Long_Long_Integer; N : Natural) return Trit_Array;

   function Bend_Stream_ASM_Recurrence
     (Unbent : Trit_Array; K_Step : Natural) return Trit_Array;

   function Apply_Bent_Circuit
     (Unbent : Bal_Trit; Bend_Step : Trit_Steps) return One_Hot_Qubit_Pair;

   function Measure_Z_Basis (Pair : One_Hot_Qubit_Pair) return Bal_Trit;

   function Verify_Bent_Ternary
     (Seed : Long_Long_Integer; Bend_K : Natural; Num_Trits : Natural)
      return Boolean;

   --  Exact ratio arithmetic for weight-space quantization scales
   function Make_Ratio (Numerator : Long_Long_Integer; Denominator : Positive)
      return Exact_Ratio with Post => Exact_Ratio'Denominator > 0;

   function Numerator (R : Exact_Ratio) return Long_Long_Integer;
   function Denominator (R : Exact_Ratio) return Positive;

   function "*" (Left, Right : Exact_Ratio) return Exact_Ratio;
   function "/" (Left, Right : Exact_Ratio) return Exact_Ratio;
   function "+" (Left, Right : Exact_Ratio) return Exact_Ratio;
   function "-" (Left, Right : Exact_Ratio) return Exact_Ratio;

   function To_Long_Long_Integer (R : Exact_Ratio) return Long_Long_Integer
     with Pre => Denominator (R) = 1;

   --  Weight-space quantization: maps u8 weight to balanced trit with exact p/q scale
   function Quantize_Weight (W : Interfaces.Unsigned_8; Scale : Exact_Ratio)
      return Bal_Trit;

private

   type Gate_Label is (Identity, Z_Cycle, X_Cycle);

   type One_Hot_Qubit_Pair is record
      Q1_Neg_Bit : Natural range 0 .. 1 := 0;
      Q0_Pos_Bit : Natural range 0 .. 1 := 0;
      Gate_Applied : Gate_Label := Identity;
   end record;

   type Exact_Ratio is record
      Numerator   : Long_Long_Integer;
      Denominator : Positive;
   end record with
     Invariant => Exact_Ratio.Denominator > 0;

end WST_Brain_Scan;
