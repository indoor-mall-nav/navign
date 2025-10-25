//
//  AppModel.swift
//  vision
//
//  Created by Ethan Goh on 10/25/25.
//

import SwiftUI

/// Maintains app-wide state
@MainActor
@Observable
class AppModel {
  let immersiveSpaceID = "ImmersiveSpace"
  enum ImmersiveSpaceState {
    case closed
    case inTransition
    case open
  }
  var immersiveSpaceState = ImmersiveSpaceState.closed
}
