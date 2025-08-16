//! LoongArch64 specific page table structures.

use core::arch::asm;

use memory_addr::VirtAddr;
use page_table_entry::loongarch64::LA64PTE;

use crate::{PageTable64, PageTable64Mut, PagingMetaData};

/// Metadata of LoongArch64 page tables.
#[derive(Copy, Clone, Debug)]
pub struct LA64MetaData;

impl LA64MetaData {
    /// PWCL(Page Walk Controller for Lower Half Address Space) CSR flags
    ///
    /// <https://loongson.github.io/LoongArch-Documentation/LoongArch-Vol1-EN.html#page-walk-controller-for-lower-half-address-space>
    ///
    /// | BitRange | Name      | Value |
    /// | ----     | ----      | ----  |
    /// | 4:0      | PTBase    |    12 |
    /// | 9:5      | PTWidth   |     9 |
    /// | 14:10    | Dir1Base  |    21 |
    /// | 19:15    | Dir1Width |     9 |
    /// | 24:20    | Dir2Base  |     0 |
    /// | 29:25    | Dir2Width |     0 |
    /// | 31:30    | PTEWidth  |     0 |
    pub const PWCL_VALUE: u32 = 12 | (9 << 5) | (21 << 10) | (9 << 15);

    /// PWCH(Page Walk Controller for Higher Half Address Space) CSR flags
    ///
    /// <https://loongson.github.io/LoongArch-Documentation/LoongArch-Vol1-EN.html#page-walk-controller-for-higher-half-address-space>
    ///
    /// | BitRange | Name                            | Value |
    /// | ----     | ----                            | ----  |
    /// | 5:0      | Dir3Base                        |    30 |
    /// | 11:6     | Dir3Width                       |     9 |
    /// | 17:12    | Dir4Base                        |     0 |
    /// | 23:18    | Dir4Width                       |     0 |
    /// | 24       | 0                               |     0 |
    /// | 24       | HPTW_En(CPUCFG.2.HPTW(bit24)=1) |     0 |
    /// | 31:25    | 0                               |     0 |
    pub const PWCH_VALUE: u32 = 30 | (9 << 6);
}

impl PagingMetaData for LA64MetaData {
    const LEVELS: usize = 3;
    const PA_MAX_BITS: usize = 40;
    const VA_MAX_BITS: usize = 40;
    type VirtAddr = VirtAddr;

    #[inline]
    fn flush_tlb(vaddr: Option<VirtAddr>) {
        unsafe {
            if let Some(vaddr) = vaddr {
                // <https://loongson.github.io/LoongArch-Documentation/LoongArch-Vol1-EN.html#_dbar>
                //
                // Only after all previous load/store access operations are completely
                // executed, the DBAR 0 instruction can be executed; and only after the
                // execution of DBAR 0 is completed, all subsequent load/store access
                // operations can be executed.
                //
                // <https://loongson.github.io/LoongArch-Documentation/LoongArch-Vol1-EN.html#_invtlb>
                //
                // formats: invtlb op, asid, addr
                //
                // op 0x5: Clear all page table entries with G=0 and ASID equal to the
                // register specified ASID, and VA equal to the register specified VA.
                //
                // When the operation indicated by op does not require an ASID, the
                // general register rj should be set to r0.
                asm!("dbar 0; invtlb 0x05, $r0, {reg}", reg = in(reg) vaddr.as_usize());
            } else {
                // op 0x0: Clear all page table entries
                asm!("dbar 0; invtlb 0x00, $r0, $r0");
            }
        }
    }
}

/// loongarch64 page table
///
/// <https://loongson.github.io/LoongArch-Documentation/LoongArch-Vol1-EN.html#section-multi-level-page-table-structure-supported-by-page-walking>
///
/// 3 levels:
///
/// using page table dir3, dir1 and pt, ignore dir4 and dir2
pub type LA64PageTable<H> = PageTable64<LA64MetaData, LA64PTE, H>;
pub type LA64PageTableMut<'a, H> = PageTable64Mut<'a, LA64MetaData, LA64PTE, H>;
