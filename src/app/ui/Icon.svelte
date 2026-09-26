<script lang="ts">
  export let name:
    | "overview"
    | "activity"
    | "apps"
    | "customize"
    | "settings"
    | "refresh"
    | "shield"
    | "pulse"
    | "alert"
    | "inbox"
    | "chevron"
    | "sparkles";
  export let size = 18;
  export let strokeWidth = 1.8;
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === "overview"}
    <rect x="3" y="3" width="7" height="7" rx="2" />
    <rect x="14" y="3" width="7" height="7" rx="2" />
    <rect x="3" y="14" width="7" height="7" rx="2" />
    <rect x="14" y="14" width="7" height="7" rx="2" />
  {:else if name === "activity"}
    <path d="M4 17l4-5 4 3 4-8 4 3" />
    <path d="M4 21h16" />
  {:else if name === "apps"}
    <rect x="3" y="4" width="18" height="16" rx="3" />
    <path d="M3 9h18M8 4v5" />
  {:else if name === "customize"}
    <path d="M12 3a3 3 0 0 0-3 3v1a2 2 0 0 1-2 2H6a3 3 0 0 0 0 6h1a2 2 0 0 1 2 2v1a3 3 0 0 0 6 0v-1a2 2 0 0 1 2-2h1a3 3 0 0 0 0-6h-1a2 2 0 0 1-2-2V6a3 3 0 0 0-3-3Z" />
    <circle cx="12" cy="12" r="2.5" />
  {:else if name === "settings"}
    <path d="M12 3.5v2M12 18.5v2M3.5 12h2M18.5 12h2M5.9 5.9l1.4 1.4M16.7 16.7l1.4 1.4M18.1 5.9l-1.4 1.4M7.3 16.7l-1.4 1.4" />
    <circle cx="12" cy="12" r="4" />
  {:else if name === "refresh"}
    <path d="M20 6v5h-5" />
    <path d="M4 18v-5h5" />
    <path d="M18.2 9A7 7 0 0 0 6.1 6.7L4 11M5.8 15A7 7 0 0 0 17.9 17.3L20 13" />
  {:else if name === "shield"}
    <path d="M12 3l7 3v5c0 4.6-2.9 8.1-7 10-4.1-1.9-7-5.4-7-10V6l7-3Z" />
    <path d="m9 12 2 2 4-4" />
  {:else if name === "pulse"}
    <path d="M3 12h4l2-5 4 10 2-5h6" />
  {:else if name === "alert"}
    <path d="M12 4 3.5 19h17L12 4Z" />
    <path d="M12 9v4M12 16.5h.01" />
  {:else if name === "inbox"}
    <path d="M4 5h16v14H4z" />
    <path d="M4 14h4l2 2h4l2-2h4" />
  {:else if name === "chevron"}
    <path d="m9 6 6 6-6 6" />
  {:else if name === "sparkles"}
    <path d="m12 3 1.2 3.1L16 7.5l-2.8 1.4L12 12l-1.2-3.1L8 7.5l2.8-1.4L12 3Z" />
    <path d="m18 13 .8 2 2.2 1-.2.1-2 1L18 19l-.8-1.9-2-1 .2-.1 1.8-.9L18 13Z" />
    <path d="m6 13 .7 1.6L8 15.3l-1.3.7L6 17.5 5.3 16 4 15.3l1.3-.7L6 13Z" />
  {/if}
</svg>
