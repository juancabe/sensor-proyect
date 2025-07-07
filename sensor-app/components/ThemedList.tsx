import React from 'react';
import {
  FlatList,
  StyleSheet,
  TouchableOpacity,
  type FlatListProps,
} from 'react-native';
import { ThemedView } from '@/components/ThemedView';
import { ThemedText } from '@/components/ThemedText';

export type ThemedListProps<T> = FlatListProps<T> & {
  items: T[];
  renderItem: (item: T) => React.ReactNode;
  onItemPress?: (item: T) => void;
  title?: string;
};

export function ThemedList<T>({
  items,
  renderItem,
  onItemPress,
  title,
  ...flatListProps
}: ThemedListProps<T>) {
  return (
    <ThemedView style={styles.container}>
      {title && (
        <ThemedText type="title" style={styles.title}>
          {title}
        </ThemedText>
      )}
      <FlatList
        extraData={items}
        keyExtractor={(item, index) => index.toString()}
        renderItem={({ item }) => (
          <TouchableOpacity
            onPress={() => onItemPress?.(item)}
            style={styles.itemContainer}
          >
            {renderItem(item)}
          </TouchableOpacity>
        )}
        scrollEnabled={false}
        {...flatListProps}
      />
    </ThemedView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 16,
  },
  title: {
    marginBottom: 16,
    fontSize: 24,
  },
  itemContainer: {
    padding: 12,
    marginBottom: 8,
    borderRadius: 8,
  },
});
