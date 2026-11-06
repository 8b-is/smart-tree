package body WST_Brain_Scan is

   function Rotate_Cyclic
     (T : Bal_Trit; Steps_Mod_3 : Natural) return Bal_Trit
   is
      Idx : constant Integer := Integer (T) + 1;
      Rotated : constant Integer :=
        (Idx + Integer (Steps_Mod_3 mod 3)) mod 3;
   begin
      return Bal_Trit (Rotated - 1);
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
      Rem : Long_Long_Integer;
      Result : Trit_Array (1 .. N);
   begin
      for I in Result'Range loop
         Rem := S mod 3;
         S := (S - Rem) / 3;
         if Rem = 2 then
            Rem := -1;
            S := S + 1;
         end if;
         Result (I) := Bal_Trit (Rem);
      end loop;
      return Result;
   end Extract_Unbent_Balanced_Trits;

   function Bend_Stream_ASM_Recurrence
     (Unbent : Trit_Array; K_Step : Natural) return Trit_Array
   is
      K_Mod3 : constant Natural := K_Step mod 3;
      R_Mod3 : Natural := 0;
      Result : Trit_Array (1 .. Unbent'Length);
      J : Natural := Result'First;
   begin
      for I in Unbent'Range loop
         Result (J) := Rotate_Cyclic (Unbent (I), R_Mod3);
         R_Mod3 := R_Mod3 + K_Mod3;
         if R_Mod3 >= 3 then
            R_Mod3 := R_Mod3 - 3;
         end if;
         J := J + 1;
      end loop;
      return Result;
   end Bend_Stream_ASM_Recurrence;

   function Apply_Bent_Circuit
     (Unbent : Bal_Trit; Bend_Step_Mod3 : Natural) return One_Hot_Qubit_Pair
   is
      Step : constant Natural := Bend_Step_Mod3 mod 3;
      Rotated : constant Bal_Trit := Rotate_Cyclic (Unbent, Step);
      Gate : Gate_Label;
      Pair : One_Hot_Qubit_Pair;
   begin
      if Step = 0 then
         Gate := Identity;
      elsif Step = 1 then
         Gate := Z_Cycle;
      else
         Gate := X_Cycle;
      end if;

      Pair.Gate_Applied := Gate;

      case Rotated is
         when -1 =>
            Pair.Q1_Neg_Bit := 1;
            Pair.Q0_Pos_Bit := 0;
         when 0 =>
            Pair.Q1_Neg_Bit := 0;
            Pair.Q0_Pos_Bit := 0;
         when 1 =>
            Pair.Q1_Neg_Bit := 0;
            Pair.Q0_Pos_Bit := 1;
      end case;

      return Pair;
   end Apply_Bent_Circuit;

   function Measure_Z_Basis (Pair : One_Hot_Qubit_Pair) return Bal_Trit is
   begin
      if Pair.Q1_Neg_Bit = 1 and Pair.Q0_Pos_Bit = 0 then
         return -1;
      elsif Pair.Q1_Neg_Bit = 0 and Pair.Q0_Pos_Bit = 1 then
         return 1;
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
      I : Natural := 1;
   begin
      for J in Unbent'Range loop
         declare
            Step : constant Natural :=
              Natural ((Long_Long_Integer (Bend_K) *
                        Long_Long_Integer (I - 1)) mod 3);
            C_Bent : constant Bal_Trit := Rotate_Cyclic (Unbent (J), Step);
            Q_Bent : constant Bal_Trit :=
              Measure_Z_Basis (Apply_Bent_Circuit (Unbent (J), Step));
         begin
            if C_Bent /= Bent (I) or else C_Bent /= Q_Bent then
               return False;
            end if;
         end;
         I := I + 1;
      end loop;
      return True;
   end Verify_Bent_Ternary;

end WST_Brain_Scan;
