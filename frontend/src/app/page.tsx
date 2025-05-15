"use client";
import { useState, useEffect, ChangeEvent, useRef } from "react";
import { update as cruxUpdate, deserializeView } from "./crux/core";
import init_core, { view as cruxView } from "shared/shared"; // Import init_core and view
import {
  ViewModel,
  FieldViewModel, // Assuming this is the generated class for fields like username, email
  EventVariantUpdateValue,
  EventVariantTouchField,
  EventVariantSetFieldEditing,
  EventVariantSubmit,
  EventVariantEdit,
  EventVariantResetForm,
  FieldIdentVariantUsername,
  FieldIdentVariantEmail,
  FieldIdentVariantAge,
  FieldIdentVariantAddress,
} from "shared_types/types/shared_types";

// Helper to create initial FieldViewModel instances
// Adjust constructor arguments if your generated FieldViewModel class is different
const createInitialField = (
  value: string,
  initialValue: string,
  touched: boolean,
  dirty: boolean,
  error: string | null,
  valid: boolean,
  editing: boolean
): FieldViewModel => {
  // Assuming FieldViewModel is a class with a constructor matching these arguments
  return new FieldViewModel(value, initialValue, touched, dirty, error, valid, editing);
};

// Create an initial ViewModel state
// This assumes ViewModel is a class with a constructor matching these arguments
const initialAppViewModel = new ViewModel(
  createInitialField("", "", false, false, "Username cannot be empty", false, false), // username
  createInitialField("", "", false, false, "Email cannot be empty", false, false),    // email
  createInitialField("", "", false, false, null, true, false),                        // age (initially valid, no error)
  createInitialField("", "", false, false, "Address cannot be empty", false, false),  // address
  false, // submitted
  true,  // is_editing_form
  "Please correct the errors.", // status_message
  false  // can_submit
);

export default function Home() {
  const [viewModel, setViewModel] = useState<ViewModel>(initialAppViewModel);
  const initialized = useRef(false);
  const justEdited = useRef(false);

  useEffect(() => {
    if (!initialized.current) {
      initialized.current = true;

      init_core().then(() => {
        // After WASM is initialized, get the current view from the core
        const rawView = cruxView();
        const currentView = deserializeView(rawView);
        setViewModel(currentView);
      });
    }
  }, []); // Run once on component mount

  const handleInputChange = (ident: FieldIdentVariantUsername | FieldIdentVariantEmail | FieldIdentVariantAge | FieldIdentVariantAddress, value: string) => {
    cruxUpdate(new EventVariantUpdateValue(ident, value), setViewModel);
  };

  const handleFieldTouch = (ident: FieldIdentVariantUsername | FieldIdentVariantEmail | FieldIdentVariantAge | FieldIdentVariantAddress) => {
    cruxUpdate(new EventVariantTouchField(ident), setViewModel);
  };

  const handleFieldFocus = (ident: FieldIdentVariantUsername | FieldIdentVariantEmail | FieldIdentVariantAge | FieldIdentVariantAddress, editing: boolean) => {
    cruxUpdate(new EventVariantSetFieldEditing(ident, editing), setViewModel);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (justEdited.current) {
      console.log("Submit event ignored immediately after edit transition.");
      return;
    }

    if (viewModel.can_submit && viewModel.is_editing_form) {
      cruxUpdate(new EventVariantSubmit(), setViewModel);
    }
  };

  const handleEdit = () => {
    console.log("View Model (before edit event)", viewModel);
    justEdited.current = true;
    cruxUpdate(new EventVariantEdit(), setViewModel);

    setTimeout(() => {
      justEdited.current = false;
    }, 0);
  };

  const handleReset = () => {
    cruxUpdate(new EventVariantResetForm(), setViewModel);
  };

  const renderField = (
    identInstance: FieldIdentVariantUsername | FieldIdentVariantEmail | FieldIdentVariantAge | FieldIdentVariantAddress,
    label: string,
    type: string = "text"
  ) => {
    // viewModel is non-null here
    let fieldKey: keyof ViewModel;
    let field: FieldViewModel; // Use the imported FieldViewModel type

    if (identInstance instanceof FieldIdentVariantUsername) {
      fieldKey = "username";
    } else if (identInstance instanceof FieldIdentVariantEmail) {
      fieldKey = "email";
    } else if (identInstance instanceof FieldIdentVariantAge) {
      fieldKey = "age";
    } else if (identInstance instanceof FieldIdentVariantAddress) {
      fieldKey = "address";
    } else {
      console.error("Unknown FieldIdent instance:", identInstance);
      return null;
    }

    // Access the field directly; TypeScript should infer its type as FieldViewModel
    // eslint-disable-next-line prefer-const
    field = viewModel[fieldKey] as FieldViewModel; 

    if (!field) return null; // Should not happen if ViewModel structure is correct

    return (
      <div className="mb-4">
        <label htmlFor={fieldKey as string} className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          {label}
        </label>
        <input
          type={type}
          id={fieldKey as string}
          name={fieldKey as string}
          value={field.value}
          onChange={(e: ChangeEvent<HTMLInputElement>) => handleInputChange(identInstance, e.target.value)}
          onBlur={() => {
            handleFieldTouch(identInstance);
            handleFieldFocus(identInstance, false);
          }}
          onFocus={() => handleFieldFocus(identInstance, true)}
          disabled={!viewModel.is_editing_form}
          className={`mt-1 block w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none sm:text-sm
                     ${field.error && field.touched ? "border-red-500 focus:ring-red-500 focus:border-red-500" : "border-gray-300 dark:border-gray-600 focus:ring-indigo-500 focus:border-indigo-500"}
                     ${!viewModel.is_editing_form ? "bg-gray-100 dark:bg-gray-700 cursor-not-allowed" : "bg-white dark:bg-gray-800 dark:text-gray-100"}`}
        />
        {field.touched && field.error && (
          <p className="mt-1 text-xs text-red-500">{field.error}</p>
        )}
      </div>
    );
  };

  // No loading state needed if viewModel is initialized directly
  // if (!viewModel) {
  //   return (
  //     <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex flex-col justify-center items-center p-4 font-[family-name:var(--font-geist-sans)]">
  //       Loading...
  //     </div>
  //   );
  // }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex flex-col justify-center items-center p-4 font-[family-name:var(--font-geist-sans)]">
      <div className="w-full max-w-md p-8 space-y-6 bg-white dark:bg-gray-800 rounded-lg shadow-md">
        <h1 className="text-2xl font-bold text-center text-gray-900 dark:text-white">
          Survey Form (Crux Powered)
        </h1>

        <form onSubmit={handleSubmit} className="space-y-6">
          {renderField(new FieldIdentVariantUsername(), "Username")}
          {renderField(new FieldIdentVariantEmail(), "Email", "email")}
          {renderField(new FieldIdentVariantAge(), "Age", "number")}
          {renderField(new FieldIdentVariantAddress(), "Address")}

          {viewModel.status_message && (
            <p className={`text-sm text-center ${viewModel.submitted && viewModel.username.valid && viewModel.email.valid && viewModel.address.valid && viewModel.age.valid ? "text-green-600" : "text-gray-600 dark:text-gray-400"}`}>
              {viewModel.status_message}
            </p>
          )}

          <div className="flex flex-col sm:flex-row gap-3">
            {!viewModel.submitted || viewModel.is_editing_form ? (
              <button
                type="submit"
                disabled={!viewModel.can_submit || !viewModel.is_editing_form}
                className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Submit
              </button>
            ) : (
              <button
                type="button"
                onClick={handleEdit}
                className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
              >
                Edit Form
              </button>
            )}
            <button
              type="button"
              onClick={handleReset}
              disabled={!viewModel.is_editing_form && viewModel.submitted}
              className="w-full flex justify-center py-2 px-4 border border-gray-300 dark:border-gray-500 rounded-md shadow-sm text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Reset
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
