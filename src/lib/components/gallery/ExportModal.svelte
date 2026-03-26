<script lang="ts">
    import { open } from "@tauri-apps/plugin-dialog";
    import {
        Button,
        BrowseInput,
        SelectionCard,
        SectionLabel,
        SplitPaneModal,
        Alert,
        LabelBadge,
        TextInput,
    } from "$lib/components/ui";

    let {
        showModal = $bindable(false),
        currentDirectoryPath = "",
        currentDatasetSummary = null,
        onclosemodal,
        onrunexport,
    }: {
        showModal?: boolean;
        currentDirectoryPath?: string;
        currentDatasetSummary?: any | null;
        onclosemodal?: () => void;
        onrunexport?: (data: {
            sourceDir: string;
            outputDir: string;
            mode: "yolo" | "labelme";
            trainRatio: number;
            valRatio: number;
            testRatio: number;
            shapeType: "polygon" | "bounding_box";
            includedLabels: string[];
        }) => void;
    } = $props();

    // Internal state
    let exportMode: "yolo" | "labelme" = $state("yolo");
    let outputDir = $state("");
    let trainRatio = $state(0.7);
    let valRatio = $state(0.2);
    let testRatio = $state(0.1);
    let shapeType: "polygon" | "bounding_box" = $state("polygon");
    let internalExcludedLabels = $state(new Set<string>());
    let localLoading = $state(false);
    let localError = $state("");

    // Reset error on open
    $effect(() => {
        if (showModal) {
            localError = "";
        }
    });

    async function selectOutputDirectory() {
        localError = "";
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Select Output Directory",
            });
            if (selected && typeof selected === "string") {
                outputDir = selected;
            }
        } catch (err) {
            localError = "Failed to select output directory.";
        }
    }

    function toggleLabelExclusion(label: string) {
        if (internalExcludedLabels.has(label)) {
            internalExcludedLabels.delete(label);
        } else {
            internalExcludedLabels.add(label);
        }
        internalExcludedLabels = internalExcludedLabels;
    }

    function handleRunExport() {
        if (!currentDirectoryPath) {
            localError = "Source directory missing.";
            return;
        }
        if (!outputDir) {
            localError = "Please select an output directory.";
            return;
        }

        const allLabels = currentDatasetSummary?.label_counts
            ? Object.keys(currentDatasetSummary.label_counts)
            : [];
        const includedLabelsArray = allLabels.filter(
            (label: string) => !internalExcludedLabels.has(label),
        );

        if (includedLabelsArray.length === 0 && allLabels.length > 0) {
            localError = "Please include at least one label.";
            return;
        }

        if (exportMode === "yolo") {
            const sum = trainRatio + valRatio + testRatio;
            if (Math.abs(sum - 1.0) > 0.015) {
                localError = `Split ratios must sum to 1.0 (Current: ${sum.toFixed(2)})`;
                return;
            }
        }
        localError = "";

        onrunexport?.({
            sourceDir: currentDirectoryPath,
            outputDir: outputDir,
            mode: exportMode,
            trainRatio,
            valRatio,
            testRatio,
            shapeType,
            includedLabels: includedLabelsArray,
        });
    }

    let availableLabels = $derived(
        currentDatasetSummary?.label_counts
            ? Object.keys(currentDatasetSummary.label_counts)
            : [],
    );
</script>

<SplitPaneModal
    show={showModal}
    title="Export Dataset"
    subtitle="Configure format & destination"
    icon="ios_share"
    onclose={() => onclosemodal?.()}
>
    {#snippet sidebar()}
    <!-- Left Sidebar -->
    <div class="flex flex-col gap-6 h-full">
        <!-- Export Format Selection -->
        <div class="flex flex-col gap-3 flex-1">
            <SectionLabel>Format</SectionLabel>

            <SelectionCard
                selected={exportMode === "yolo"}
                color="primary"
                onselect={() => (exportMode = "yolo")}
            >
                {#snippet icon()}<span class="font-bold text-xs">YOLO</span>{/snippet}
                {#snippet title()}<span>YOLO Format</span>{/snippet}
                {#snippet description()}<span>For detection models</span>{/snippet}
            </SelectionCard>

            <SelectionCard
                selected={exportMode === "labelme"}
                color="secondary"
                onselect={() => (exportMode = "labelme")}
            >
                {#snippet icon()}<span class="material-symbols-rounded text-sm"
                    >data_object</span
                >{/snippet}
                {#snippet title()}<span>LabelMe JSON</span>{/snippet}
                {#snippet description()}<span>Raw annotations</span>{/snippet}
            </SelectionCard>
        </div>

        <!-- Directories -->
        <div class="space-y-3">
            <SectionLabel>Paths</SectionLabel>

            <!-- Source Path (Compact) -->
            <div
                class="tooltip tooltip-right w-full"
                data-tip={currentDirectoryPath}
            >
                <div
                    class="flex items-center gap-2 px-3 py-2 bg-base-200 rounded-lg text-xs opacity-70"
                >
                    <span
                        class="material-symbols-rounded text-base-content/50 text-sm"
                        >folder_open</span
                    >
                    <span class="truncate"
                        >{currentDirectoryPath.split("/").pop()}</span
                    >
                </div>
            </div>

            <!-- Output Path -->
            <BrowseInput
                value={outputDir}
                placeholder="Select output..."
                icon="output"
                onbrowse={selectOutputDirectory}
            />
        </div>
    </div>
    {/snippet}

    {#snippet content()}
    <!-- Main Content -->
    <div class="space-y-8">
        {#if localError}
            <Alert
                variant="error"
                dismissible
                onclose={() => (localError = "")}
            >
                {localError}
            </Alert>
        {/if}

        <!-- Section: Configuration Details -->
        {#if exportMode === "yolo"}
            <div class="animate-in fade-in zoom-in-95 duration-200">
                <SectionLabel>Configuration</SectionLabel>

                <div
                    class="mt-4 grid grid-cols-2 gap-6 bg-base-200/30 p-5 rounded-xl border border-base-200"
                >
                    <!-- Shape Type -->
                    <div class="col-span-2">
                        <span
                            class="text-xs font-semibold text-base-content/60 mb-2 block"
                            >Annotation Type</span
                        >
                        <div class="join w-full grid grid-cols-2 h-9">
                            <button
                                class={`join-item btn btn-sm border-base-300 font-normal ${shapeType === "polygon" ? "btn-active btn-primary text-white" : "bg-base-100"}`}
                                onclick={() => (shapeType = "polygon")}
                            >
                                Polygon
                            </button>
                            <button
                                class={`join-item btn btn-sm border-base-300 font-normal ${shapeType === "bounding_box" ? "btn-active btn-primary text-white" : "bg-base-100"}`}
                                onclick={() => (shapeType = "bounding_box")}
                            >
                                Bounding Box
                            </button>
                        </div>
                    </div>

                    <!-- Split Ratios -->
                    <div class="col-span-2">
                        <span
                            class="text-xs font-semibold text-base-content/60 mb-2 block"
                            >Dataset Split</span
                        >
                        <div class="flex items-center gap-3">
                            <div
                                class="flex-1 flex flex-col items-center gap-1"
                            >
                                <TextInput
                                    type="number"
                                    size="xs"
                                    class="w-full text-center font-mono"
                                    bind:value={trainRatio}
                                    step="0.1"
                                />
                                <div
                                    class="w-full h-1 bg-green-500 rounded-full opacity-60"
                                ></div>
                                <span
                                    class="text-[10px] uppercase font-bold text-base-content/40"
                                    >Train</span
                                >
                            </div>
                            <div
                                class="flex-1 flex flex-col items-center gap-1"
                            >
                                <TextInput
                                    type="number"
                                    size="xs"
                                    class="w-full text-center font-mono"
                                    bind:value={valRatio}
                                    step="0.1"
                                />
                                <div
                                    class="w-full h-1 bg-blue-500 rounded-full opacity-60"
                                ></div>
                                <span
                                    class="text-[10px] uppercase font-bold text-base-content/40"
                                    >Val</span
                                >
                            </div>
                            <div
                                class="flex-1 flex flex-col items-center gap-1"
                            >
                                <TextInput
                                    type="number"
                                    size="xs"
                                    class="w-full text-center font-mono"
                                    bind:value={testRatio}
                                    step="0.1"
                                />
                                <div
                                    class="w-full h-1 bg-purple-500 rounded-full opacity-60"
                                ></div>
                                <span
                                    class="text-[10px] uppercase font-bold text-base-content/40"
                                    >Test</span
                                >
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        {:else}
            <div
                class="animate-in fade-in zoom-in-95 duration-200 h-32 flex items-center justify-center bg-base-200/30 rounded-xl border border-base-200 border-dashed"
            >
                <div class="text-center opacity-50">
                    <span class="material-symbols-rounded text-3xl"
                        >data_object</span
                    >
                    <p class="text-sm mt-1">Extracts raw LabelMe JSON files</p>
                </div>
            </div>
        {/if}

        <!-- Section: Class Filtering -->
        <div>
            <div class="flex justify-between items-center mb-4">
                <SectionLabel>Classes</SectionLabel>
                <span class="text-xs text-base-content/40">
                    {availableLabels.filter(
                        (l) => !internalExcludedLabels.has(l),
                    ).length} selected
                </span>
            </div>

            {#if availableLabels.length > 0}
                <div class="flex flex-wrap gap-2">
                    {#each availableLabels as label}
                        <LabelBadge
                            {label}
                            count={currentDatasetSummary?.label_counts[label]}
                            state={!internalExcludedLabels.has(label)
                                ? "active"
                                : "excluded"}
                            onclick={() => toggleLabelExclusion(label)}
                        />
                    {/each}
                </div>
            {:else}
                <div
                    class="text-center py-6 text-base-content/40 text-sm italic"
                >
                    No classes found in dataset.
                </div>
            {/if}
        </div>
    </div>
    {/snippet}

    {#snippet footer()}
    <!-- Footer -->
    <div class="flex gap-3">
        <Button variant="ghost" onclick={() => onclosemodal?.()}
            >Cancel</Button
        >
        <Button
            onclick={handleRunExport}
            disabled={localLoading || !outputDir}
            minWidth="140px"
        >
            {#if localLoading}
                <span class="loading loading-spinner loading-xs"></span>
            {:else}
                Export Data
            {/if}
        </Button>
    </div>
    {/snippet}
</SplitPaneModal>
