<script lang="ts">
  let { data, type } = $props<{
    data: ArrayBuffer;
    type: "docx" | "xlsx";
  }>();

  let containerEl: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (!containerEl) return;

    (async () => {
      if (type === "docx") {
        const docxPreview = await import("docx-preview");
        containerEl.innerHTML = "";
        await docxPreview.renderAsync(data, containerEl);
      } else if (type === "xlsx") {
        const XLSX = await import("xlsx");
        const wb = XLSX.read(data);
        const firstSheet = wb.SheetNames[0];
        if (firstSheet) {
          const html = XLSX.utils.sheet_to_html(wb.Sheets[firstSheet]);
          containerEl.innerHTML = html;
        }
      }
    })();
  });
</script>

<div bind:this={containerEl} class="h-full overflow-auto p-4 office-preview"></div>

<style>
  .office-preview :global(table) {
    border-collapse: collapse;
    width: 100%;
  }
  .office-preview :global(td), .office-preview :global(th) {
    border: 1px solid var(--color-border);
    padding: 4px 8px;
    font-size: 0.875rem;
  }
</style>
