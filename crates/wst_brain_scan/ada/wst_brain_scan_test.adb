with Ada.Text_IO;
with Interfaces;
with WST_Brain_Scan;

procedure WST_Brain_Scan_Test is
   Diff_Failed : exception;
begin
   if not WST_Brain_Scan.Verify_Bent_Ternary (19952026, 2, 14) then
      raise Diff_Failed;
   end if;

   if WST_Brain_Scan.Ternary_NAND
        (WST_Brain_Scan.Bal_Trit'Last, WST_Brain_Scan.Bal_Trit'Last)
     /= WST_Brain_Scan.Bal_Trit'First
   then
      raise Diff_Failed;
   end if;

   --  Test Exact_Ratio arithmetic
   declare
      R1 : constant WST_Brain_Scan.Exact_Ratio :=
        WST_Brain_Scan.Make_Ratio (1, 3);
      R2 : constant WST_Brain_Scan.Exact_Ratio :=
        WST_Brain_Scan.Make_Ratio (3, 2);
      R3 : constant WST_Brain_Scan.Exact_Ratio := R1 * R2;
      R4 : constant WST_Brain_Scan.Exact_Ratio := R2 / R1;
   begin
      --  (1/3) * (3/2) = 1/2
      if WST_Brain_Scan.Numerator (R3) /= 1 or WST_Brain_Scan.Denominator (R3) /= 2 then
         raise Diff_Failed;
      end if;

      --  (3/2) / (1/3) = 9/2
      if WST_Brain_Scan.Numerator (R4) /= 9 or WST_Brain_Scan.Denominator (R4) /= 2 then
         raise Diff_Failed;
      end if;

      --  Test Quantize_Weight with exact scale
      declare
         Scale : constant WST_Brain_Scan.Exact_Ratio :=
           WST_Brain_Scan.Make_Ratio (1, 1);
         Q : constant WST_Brain_Scan.Bal_Trit :=
           WST_Brain_Scan.Quantize_Weight (Interfaces.Unsigned_8 (50), Scale);
      begin
         if Q /= 0 then
            raise Diff_Failed;
         end if;
      end;
   end;

   Ada.Text_IO.Put_Line ("Ada 2022 lane: agreed 14, disagreed 0");
end WST_Brain_Scan_Test;
