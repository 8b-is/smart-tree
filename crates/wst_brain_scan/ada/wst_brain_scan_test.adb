with Ada.Text_IO; use Ada.Text_IO;
with WST_Brain_Scan; use WST_Brain_Scan;

procedure WST_Brain_Scan_Test is
begin
   if not Verify_Bent_Ternary (19952026, 2, 14) then
      raise Program_Error with "Ada 2022 bent ternary cross-layer diff failed";
   end if;

   Put_Line ("Ada 2022 lane: agreed 14, disagreed 0");
end WST_Brain_Scan_Test;
