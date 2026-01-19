//
//  ContentView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct ContentView: View {
    var body: some View {
        VStack {
            Image(systemName: "pawprint.fill")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text("Welcome to OFFLEASH")
        }
        .padding()
    }
}

#Preview {
    ContentView()
}
