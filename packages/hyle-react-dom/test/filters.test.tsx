import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import {
  FilterString,
  FilterNumber,
  FilterBoolean,
  FilterReference,
  FilterArray,
  FilterFile,
} from "../src/components/filters";
import type { FilterProps } from "@tty-pt/hyle-react";
import type { Result, Row } from "@tty-pt/hyle-react";

function makeProps(overrides: Partial<FilterProps<string>> = {}): FilterProps<string> {
  return {
    label: "Field",
    value: "",
    field: { label: "Field", type: { kind: "primitive", primitive: "string" } },
    fieldName: "field",
    result: null,
    onChange: vi.fn(),
    ...overrides,
  };
}

function makeReferenceProps(
  overrides: Partial<FilterProps<string>> = {},
  lookups: Record<string, Row> = {},
): FilterProps<string> {
  const result: Result = {
    total: 0,
    rows: [],
    lookups: { role: lookups },
  };
  return {
    label: "Role",
    value: "",
    field: {
      label: "Role",
      type: { kind: "reference", reference: { entity: "role", displayField: "name" } },
    },
    fieldName: "role",
    result,
    onChange: vi.fn(),
    ...overrides,
  };
}

// ── FilterString ──────────────────────────────────────────────────────────────

describe("FilterString", () => {
  it("renders a text input with the label as placeholder", () => {
    render(<FilterString {...makeProps({ label: "Name" })} />);
    const input = screen.getByRole("textbox", { name: "Name" });
    expect((input as HTMLInputElement).type).toBe("text");
  });

  it("calls onChange with the new value on each keystroke", async () => {
    const onChange = vi.fn();
    render(<FilterString {...makeProps({ onChange })} />);
    await userEvent.type(screen.getByRole("textbox"), "Alice");
    expect(onChange).toHaveBeenCalledTimes(5);
    expect(onChange).toHaveBeenNthCalledWith(1, "A");
    expect(onChange).toHaveBeenNthCalledWith(5, "e");
  });

  it("reflects the value prop", () => {
    render(<FilterString {...makeProps({ value: "hello" })} />);
    expect((screen.getByRole("textbox") as HTMLInputElement).value).toBe("hello");
  });
});

// ── FilterNumber ──────────────────────────────────────────────────────────────

describe("FilterNumber", () => {
  it("renders a number input", () => {
    render(<FilterNumber {...makeProps({ label: "Age" })} />);
    expect((screen.getByRole("spinbutton", { name: "Age" }) as HTMLInputElement).type).toBe("number");
  });

  it("calls onChange with string value on each keystroke", async () => {
    const onChange = vi.fn();
    render(<FilterNumber {...makeProps({ onChange })} />);
    await userEvent.type(screen.getByRole("spinbutton"), "42");
    expect(onChange).toHaveBeenCalledTimes(2);
    expect(onChange).toHaveBeenNthCalledWith(1, "4");
    expect(onChange).toHaveBeenNthCalledWith(2, "2");
  });
});

// ── FilterBoolean ─────────────────────────────────────────────────────────────

function makeBoolProps(overrides: Partial<FilterProps<boolean | undefined>> = {}): FilterProps<boolean | undefined> {
  return {
    label: "Field",
    value: undefined,
    field: { label: "Field", type: { kind: "primitive", primitive: "boolean" } },
    fieldName: "field",
    result: null,
    onChange: vi.fn(),
    ...overrides,
  };
}

describe("FilterBoolean — default (checkbox)", () => {
  it("renders a checkbox", () => {
    render(<FilterBoolean {...makeBoolProps({ label: "Active" })} appearance="checkbox" context="filter" />);
    expect(screen.getByRole("checkbox", { name: "Active" })).toBeDefined();
  });

  it("is unchecked when value is undefined", () => {
    render(<FilterBoolean {...makeBoolProps({ value: undefined })} appearance="checkbox" context="filter" />);
    expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(false);
  });

  it("is checked when value is true", () => {
    render(<FilterBoolean {...makeBoolProps({ value: true })} appearance="checkbox" context="filter" />);
    expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(true);
  });

  it("calls onChange with true when checked", async () => {
    const onChange = vi.fn();
    render(<FilterBoolean {...makeBoolProps({ value: undefined, onChange })} appearance="checkbox" context="filter" />);
    await userEvent.click(screen.getByRole("checkbox"));
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("calls onChange with undefined when unchecked", async () => {
    const onChange = vi.fn();
    render(<FilterBoolean {...makeBoolProps({ value: true, onChange })} appearance="checkbox" context="filter" />);
    await userEvent.click(screen.getByRole("checkbox"));
    expect(onChange).toHaveBeenCalledWith(undefined);
  });
});

describe("FilterBoolean — appearance='select'", () => {
  it("renders a select with Any/Yes/No options", () => {
    render(<FilterBoolean {...makeBoolProps({ label: "Active" })} appearance="select" context="filter" />);
    const select = screen.getByRole("combobox", { name: "Active" });
    expect(select).toBeDefined();
    expect(screen.getByRole("option", { name: "Any" })).toBeDefined();
    expect(screen.getByRole("option", { name: "Yes" })).toBeDefined();
    expect(screen.getByRole("option", { name: "No"  })).toBeDefined();
  });

  it("calls onChange with true when Yes is selected", async () => {
    const onChange = vi.fn();
    render(<FilterBoolean {...makeBoolProps({ onChange })} appearance="select" context="filter" />);
    await userEvent.selectOptions(screen.getByRole("combobox"), "Yes");
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("calls onChange with false when No is selected", async () => {
    const onChange = vi.fn();
    render(<FilterBoolean {...makeBoolProps({ onChange })} appearance="select" context="filter" />);
    await userEvent.selectOptions(screen.getByRole("combobox"), "No");
    expect(onChange).toHaveBeenCalledWith(false);
  });

  it("calls onChange with undefined when Any is selected", async () => {
    const onChange = vi.fn();
    render(<FilterBoolean {...makeBoolProps({ value: true, onChange })} appearance="select" context="filter" />);
    await userEvent.selectOptions(screen.getByRole("combobox"), "Any");
    expect(onChange).toHaveBeenCalledWith(undefined);
  });
});

// ── FilterReference ───────────────────────────────────────────────────────────

describe("FilterReference — default (select)", () => {
  it("renders a select with Any + lookup options", () => {
    const props = makeReferenceProps({}, {
      "1": { id: "1", name: "Admin" },
      "2": { id: "2", name: "Viewer" },
    });
    render(<FilterReference {...props} />);
    const select = screen.getByRole("combobox", { name: "Role" });
    expect(select).toBeDefined();
    expect(screen.getByRole("option", { name: "Any" })).toBeDefined();
    expect(screen.getByRole("option", { name: "Admin" })).toBeDefined();
    expect(screen.getByRole("option", { name: "Viewer" })).toBeDefined();
  });

  it("calls onChange with the selected option id", async () => {
    const onChange = vi.fn();
    const props = makeReferenceProps({ onChange }, {
      "1": { id: "1", name: "Admin" },
    });
    render(<FilterReference {...props} />);
    await userEvent.selectOptions(screen.getByRole("combobox"), "Admin");
    expect(onChange).toHaveBeenCalledWith("1");
  });

  it("renders a disabled Loading… select when result is null", () => {
    const props = makeReferenceProps({ result: null });
    render(<FilterReference {...props} />);
    const select = screen.getByRole("combobox") as HTMLSelectElement;
    expect(select.disabled).toBe(true);
    const options = screen.getAllByRole("option");
    expect(options).toHaveLength(1);
    expect(options[0].textContent).toBe("Loading…");
  });
});

describe("FilterReference — appearance='autocomplete'", () => {
  it("renders a text input with combobox role", () => {
    const props = makeReferenceProps({}, { "1": { id: "1", name: "Admin" } });
    render(<FilterReference {...props} appearance="autocomplete" />);
    expect(screen.getByRole("combobox", { name: "Role" })).toBeDefined();
  });

  it("renders a datalist with lookup options", () => {
    const props = makeReferenceProps({}, {
      "1": { id: "1", name: "Admin" },
      "2": { id: "2", name: "Viewer" },
    });
    const { container } = render(<FilterReference {...props} appearance="autocomplete" />);
    const datalist = container.querySelector("datalist");
    expect(datalist).toBeDefined();
    const options = datalist!.querySelectorAll("option");
    expect(options).toHaveLength(2);
  });

  it("calls onChange on input", async () => {
    const onChange = vi.fn();
    const props = makeReferenceProps({ onChange }, { "1": { id: "1", name: "Admin" } });
    render(<FilterReference {...props} appearance="autocomplete" />);
    await userEvent.type(screen.getByRole("combobox", { name: "Role" }), "Ad");
    expect(onChange).toHaveBeenCalledTimes(2);
  });
});

// ── FilterFile ────────────────────────────────────────────────────────────────

describe("FilterFile — context='filter' (default)", () => {
  it("renders a text input", () => {
    render(<FilterFile {...makeProps({ label: "File" })} context="filter" />);
    expect((screen.getByRole("textbox", { name: "File" }) as HTMLInputElement).type).toBe("text");
  });
});

describe("FilterFile — context='form'", () => {
  it("renders a file input", () => {
    render(<FilterFile {...makeProps({ label: "Upload" })} context="form" />);
    expect((screen.getByLabelText("Upload") as HTMLInputElement).type).toBe("file");
  });

  it("accept attribute is passed", () => {
    const props = makeProps({
      field: {
        label: "Images",
        type: { kind: "primitive", primitive: "file" },
        options: { accept: "image/*" },
      },
      context: "form",
    });
    render(<FilterFile {...props} />);
    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    expect(input?.accept).toBe("image/*");
  });

  it("multiple attribute is passed", () => {
    const props = makeProps({
      field: {
        label: "Files",
        type: { kind: "primitive", primitive: "file" },
        options: { multiple: true },
      },
      context: "form",
    });
    render(<FilterFile {...props} />);
    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    expect(input?.multiple).toBe(true);
  });

  it("required attribute is passed", () => {
    const props = makeProps({
      field: {
        label: "Document",
        type: { kind: "primitive", primitive: "file" },
        options: { required: true },
      },
      context: "form",
    });
    render(<FilterFile {...props} />);
    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    expect(input?.required).toBe(true);
  });
});

describe("FilterBoolean — context='form'", () => {
  it("renders a checkbox by default (not select)", () => {
    render(<FilterBoolean {...makeBoolProps({ label: "Enabled" })} context="form" />);
    expect(screen.getByRole("checkbox", { name: "Enabled" })).toBeDefined();
    expect(screen.queryByRole("combobox")).toBeNull();
  });

  it("still respects explicit appearance prop", () => {
    render(<FilterBoolean {...makeBoolProps({ label: "Status" })} appearance="select" context="form" />);
    expect(screen.getByRole("combobox", { name: "Status" })).toBeDefined();
  });
});

// ── FilterArray ───────────────────────────────────────────────────────────────

function makeArrayProps(
  overrides: Partial<FilterProps<string>> = {},
  lookups: Record<string, Row> = {},
): FilterProps<string> {
  const result: Result = {
    total: 0,
    rows: [],
    lookups: { tag: lookups },
  };
  return {
    label: "Tags",
    value: "",
    field: {
      label: "Tags",
      type: { kind: "array", item: { kind: "reference", reference: { entity: "tag", displayField: "name" } } },
    },
    fieldName: "tags",
    result,
    onChange: vi.fn(),
    ...overrides,
  };
}

describe("FilterArray", () => {
  it("renders a fieldset with checkboxes for each lookup option", () => {
    const props = makeArrayProps({}, {
      rust: { id: "rust", name: "Rust" },
      web:  { id: "web",  name: "Web"  },
    });
    render(<FilterArray {...props} />);
    expect(screen.getByRole("group", { name: "Tags" })).toBeDefined();
    expect(screen.getByRole("checkbox", { name: /Rust/ })).toBeDefined();
    expect(screen.getByRole("checkbox", { name: /Web/ })).toBeDefined();
  });

  it("calls onChange with an array containing the selected option id when checked", async () => {
    const onChange = vi.fn();
    const props = makeArrayProps({ onChange, value: "" as unknown as string }, {
      rust: { id: "rust", name: "Rust" },
    });
    render(<FilterArray {...props} />);
    await userEvent.click(screen.getByRole("checkbox", { name: /Rust/ }));
    expect(onChange).toHaveBeenCalledWith(["rust"]);
  });

  it("renders Loading… when result is null", () => {
    const props = makeArrayProps({ result: null });
    render(<FilterArray {...props} />);
    expect(screen.getByText("Loading…")).toBeDefined();
  });
});
