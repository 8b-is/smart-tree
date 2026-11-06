package body WST_Brain_Scan is

   Base_3 : constant := 3;
   Trit_Offset : constant := 1;
   Base_3_Top_Digit : constant := 2;
   Balanced_Minus_One : constant Long_Long_Integer := -1;

   function Gcd (A, B : Long_Long_Integer) return Long_Long_Integer is
         A_Abs : Long_Long_Integer := A'abs;
         B_Abs : Long_Long_Integer := B'abs;
         Remainder : Long_Long_Integer;
      begin
         while B_Abs /= 0 loop
               Remainder := A_Abs mod B_Abs;
               A_Abs := B_Abs;
               B_Abs := Remainder;
         end loop;
         return A_Abs;
      end Gcd;

   function Normalize (Num : Long_Long_Integer; Den : Positive) return Exact_Ratio is
      G : constant Long_Long_Integer := Gcd (Num, Long_Long_Integer (Den));
      N : constant Long_Long_Integer := Num / G;
      D : constant Positive := Positive (Long_Long_Integer (Den) / G);
   begin
      return (Numerator => N, Denominator => D);
   end Normalize;

   function Rotate_Cyclic (T : Bal_Trit; Steps : Trit_Steps) return Bal_Trit is
      Idx : constant Integer := Integer (T) + Trit_Offset;
      Rotated : constant Integer := (Idx + Integer (Steps)) mod Base_3;
   begin
      return Bal_Trit (Rotated - Trit_Offset);
   end Rotate_Cyclic;

   function Ternary_NAND (A, B : Bal_Trit) return Bal_Trit is
      Min_AB : constant Bal_Trit := (if A < B then A else B);
   begin
      return -Min_AB;
   end Ternary_NAND;

   function Extract_Unbent_Balanced_Trits
     (Seed : Long_Long_Integer; N : Natural) return Trit_Array
   is
      S : Long_Long_Integer := Seed;
      Remainder : Long_Long_Integer;
      Result : Trit_Array (1 .. N);
   begin
      for I in Result'Range loop
         Remainder := S mod Base_3;
         S := (S - Remainder) / Base_3;
         if Remainder = Base_3_Top_Digit then
            Remainder := Balanced_Minus_One;
            S := S + Trit_Offset;
         end if;
         Result (I) := Bal_Trit (Remainder);
      end loop;
      return Result;
   end Extract_Unbent_Balanced_Trits;

   function Bend_Stream_ASM_Recurrence
     (Unbent : Trit_Array; K_Step : Natural) return Trit_Array
   is
      K_Mod3 : constant Natural := K_Step mod Base_3;
      R_Mod3 : Natural := 0;
      Result : Trit_Array (1 .. Unbent'Length);
   begin
      for I in Unbent'Range loop
         Result (Result'First + (I - Unbent'First)) :=
           Rotate_Cyclic (Unbent (I), Trit_Steps (R_Mod3));
         R_Mod3 := R_Mod3 + K_Mod3;
         if R_Mod3 >= Base_3 then
            R_Mod3 := R_Mod3 - Base_3;
         end if;
      end loop;
      return Result;
   end Bend_Stream_ASM_Recurrence;

   function Apply_Bent_Circuit
     (Unbent : Bal_Trit; Bend_Step : Trit_Steps) return One_Hot_Qubit_Pair
   is
      Rotated : constant Bal_Trit := Rotate_Cyclic (Unbent, Bend_Step);
      Gate : constant Gate_Label :=
        (case Bend_Step is
            when 0 => Identity,
            when 1 => Z_Cycle,
            when 2 => X_Cycle);
   begin
      return
        (case Rotated is
            when -1 =>
              (Q1_Neg_Bit => 1, Q0_Pos_Bit => 0, Gate_Applied => Gate),
            when 0 =>
              (Q1_Neg_Bit => 0, Q0_Pos_Bit => 0, Gate_Applied => Gate),
            when 1 =>
              (Q1_Neg_Bit => 0, Q0_Pos_Bit => 1, Gate_Applied => Gate));
   end Apply_Bent_Circuit;

   function Measure_Z_Basis (Pair : One_Hot_Qubit_Pair) return Bal_Trit is
   begin
      if Pair.Q1_Neg_Bit = 1 and then Pair.Q0_Pos_Bit = 0 then
         return Bal_Trit'First;
      elsif Pair.Q1_Neg_Bit = 0 and then Pair.Q0_Pos_Bit = 1 then
         return Bal_Trit'Last;
      else
         return 0;
      end if;
   end Measure_Z_Basis;

   function Verify_Bent_Ternary
     (Seed : Long_Long_Integer; Bend_K : Natural; Num_Trits : Natural)
      return Boolean
   is
      Unbent : constant Trit_Array :=
        Extract_Unbent_Balanced_Trits (Seed, Num_Trits);
      Bent : constant Trit_Array :=
        Bend_Stream_ASM_Recurrence (Unbent, Bend_K);
   begin
      for I in Unbent'Range loop
         declare
            Step : constant Trit_Steps :=
              Trit_Steps ((Bend_K * (I - Unbent'First)) mod Base_3);
            C_Bent : constant Bal_Trit := Rotate_Cyclic (Unbent (I), Step);
            Q_Bent : constant Bal_Trit :=
              Measure_Z_Basis (Apply_Bent_Circuit (Unbent (I), Step));
         begin
            if C_Bent /= Bent (I) or else C_Bent /= Q_Bent then
               return False;
            end if;
         end;
      end loop;
      return True;
   end Verify_Bent_Ternary;

   function Make_Ratio (Numerator : Long_Long_Integer; Denominator : Positive)
      return Exact_Ratio
   is
   begin
      return Normalize (Numerator, Denominator);
   end Make_Ratio;

   function Numerator (R : Exact_Ratio) return Long_Long_Integer is
      (R.Numerator);

   function Denominator (R : Exact_Ratio) return Positive is
      (R.Denominator);

   function "*" (Left, Right : Exact_Ratio) return Exact_Ratio is
      (Normalize (Left.Numerator * Right.Numerator,
                  Positive (Long_Long_Integer (Left.Denominator) * Right.Denominator)));

   function "/" (Left, Right : Exact_Ratio) return Exact_Ratio is
      (Normalize (Left.Numerator * Long_Long_Integer (Right.Denominator),
                  Positive (Long_Long_Integer (Left.Denominator) * Right.Numerator'abs)));

   function "+" (Left, Right : Exact_Ratio) return Exact_Ratio is
      (Normalize (Left.Numerator * Long_Long_Integer (Right.Denominator) +
                  Right.Numerator * Long_Long_Integer (Left.Denominator),
                  Positive (Long_Long_Integer (Left.Denominator) * Right.Denominator)));

   function "-" (Left, Right : Exact_Ratio) return Exact_Ratio is
      (Normalize (Left.Numerator * Long_Long_Integer (Right.Denominator) -
                  Right.Numerator * Long_Long_Integer (Left.Denominator),
                  Positive (Long_Long_Integer (Left.Denominator) * Right.Denominator)));

   function To_Long_Long_Integer (R : Exact_Ratio) return Long_Long_Integer is
      (R.Numerator / Long_Long_Integer (R.Denominator));

   function Quantize_Weight (W : Interfaces.Unsigned_8; Scale : Exact_Ratio)
      return Bal_Trit
   is
      --  Exact p/q scale: threshold_1 = 85 * p/q, threshold_2 = 170 * p/q
      --  Using centered ternary: 0 -> -1, 1 -> 0, 2 -> +1
      T1 : constant Long_Long_Integer := (85 * Scale.Numerator) / Long_Long_Integer (Scale.Denominator);
      T2 : constant Long_Long_Integer := (170 * Scale.Numerator) / Long_Long_Integer (Scale.Denominator);
      W_Val : constant Long_Long_Integer := Long_Long_Integer (W);
   begin
      if W_Val < T1 then
         return -1;
      elsif W_Val < T2 then
         return 0;
      else
         return 1;
      end if;
   end Quantize_Weight;

end WST_Brain_Scan;
