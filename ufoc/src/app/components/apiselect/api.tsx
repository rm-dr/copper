import { Select } from "@mantine/core";
import { useEffect, useState } from "react";

type SelectorOption = {
	label: string;
	value: string;
	disabled: boolean;
};

type SelectorData = {
	error: boolean;
	loading: boolean;
	options: SelectorOption[] | null;
};

// A dropdown that fetches its options from an api
// (or any other generating function)
export function ApiSelector<T>(params: {
	onSelect: (value: string | null) => void;

	// Parameters passed to `update_list`
	update_params: T;

	error?: boolean;

	// Called whenever we need to update this selector's list of options.
	// If this returns `null`, we assume that we don't have enough information
	// to fetch options (e.g, we need a value from another selector)
	update_list: (update_params: T) => Promise<SelectorOption[] | null>;

	messages: {
		// Message to show when search finds no options,
		// but options exist
		nothingmsg_normal: string;

		// Message to show when we successfully load an empty
		// list of options
		nothingmsg_empty: string;

		// Placeholder to show when option fetch fails
		placeholder_error: string;

		// Placeholder to show when everything is normal
		placeholder_normal: string;

		// Used as placeholder AND nothingmsg whenever `update_list` returns `null`
		message_null?: string;

		// Used as placeholder while data is loading
		message_loading: string;
	};
}) {
	const [selectorState, setSelectorState] = useState<SelectorData>({
		error: false,
		loading: false,
		options: null,
	});

	// Make sure useEffect runs only when it has to
	const update_params = params.update_params;
	const update_list = params.update_list;

	// Update options right away.
	// Refresh every n seconds to detect
	// (and recover from) server disconnect.
	useEffect(() => {
		const update_options = () => {
			setSelectorState((s) => ({
				...s,
				error: false,
				loading: true,
			}));

			update_list(update_params)
				?.then((data) => {
					setSelectorState({
						options: data,
						error: false,
						loading: false,
					});
				})
				.catch(() => {
					setSelectorState({
						options: null,
						loading: false,
						error: true,
					});
				});
		};

		update_options();
		const id = setInterval(update_options, 10_000);
		return () => clearInterval(id);
	}, [update_params, update_list]);
	return (
		<Select
			onChange={params.onSelect}
			onClear={() => {
				params.onSelect(null);
			}}
			nothingFoundMessage={
				selectorState.loading && selectorState.options === null
					? params.messages.message_loading
					: selectorState.options !== null
					? selectorState.options.length != 0
						? params.messages.nothingmsg_normal
						: params.messages.nothingmsg_empty
					: params.messages.message_null
			}
			placeholder={
				selectorState.error
					? params.messages.placeholder_error
					: selectorState.options !== null
					? params.messages.placeholder_normal
					: params.messages.message_null
			}
			data={selectorState.options === null ? [] : selectorState.options}
			comboboxProps={{
				transitionProps: { transition: "fade-down", duration: 200 },
			}}
			error={selectorState.error || params.error === true}
			disabled={selectorState.error || selectorState.options === null}
			searchable
			clearable
		/>
	);
}
